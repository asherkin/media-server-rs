use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use futures::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use warp::ws::Message;
use warp::Filter;

use media_server::sdp::attributes::Candidate;
use media_server::sdp::enums::{FingerprintHashFunction, IceCandidateType, IceTransportType, MediaType, RtpCodecName};
use media_server::sdp::types::CertificateFingerprint;
use media_server::sdp::webrtc::{RtpEncoding, RtpMediaDescription, UnifiedBundleSession};
use media_server::{
    DtlsConnectionHash, LoggingLevel, MediaFrameType, Properties, RtpBundleTransport, RtpBundleTransportConnection,
    RtpIncomingSourceGroup,
};

#[derive(Debug, Clone, StructOpt)]
struct Opts {
    #[structopt(short, long, default_value = "127.0.0.1:8080")]
    listen: SocketAddr,
    #[structopt(short, long, default_value = "127.0.0.1")]
    public_ip: IpAddr,
    #[structopt(short = "r", long, parse(try_from_str = parse_port_range))]
    port_range: Option<(u16, u16)>,
}

fn parse_port_range(s: &str) -> Result<(u16, u16), String> {
    let split_point = s.find('-').ok_or("expected 'min-max' range")?;
    let min = u16::from_str(&s[..split_point]).map_err(|e| e.to_string())?;
    let max = u16::from_str(&s[split_point + 1..]).map_err(|e| e.to_string())?;
    Ok((min, max))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum C2SMessage {
    Offer {
        #[serde(with = "serde_with::rust::display_fromstr")]
        sdp: UnifiedBundleSession,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum S2CMessage {
    Answer {
        #[serde(with = "serde_with::rust::display_fromstr")]
        sdp: UnifiedBundleSession,
    },
}

async fn send_message(websocket: &mut warp::ws::WebSocket, message: &S2CMessage) -> Result<(), Box<dyn Error>> {
    let message = serde_json::to_string(message).unwrap();

    log::info!("sending: {}", message);

    websocket.send(Message::text(message)).await?;

    Ok(())
}

fn add_rtp_properties_from_media_description(properties: &mut Properties, media_description: &RtpMediaDescription) {
    let kind = &media_description.kind;

    for (i, payload) in media_description.payloads.iter().enumerate() {
        properties.set_string(&format!("{}.codecs.{}.codec", kind, i), payload.name.as_ref());
        properties.set_int(&format!("{}.codecs.{}.pt", kind, i), payload.payload_type.0 as i32);

        if let Some(rtx_payload_type) = payload.rtx_payload_type {
            properties.set_int(&format!("{}.codecs.{}.rtx", kind, i), rtx_payload_type.0 as i32);
        }
    }

    properties.set_int(
        &format!("{}.codecs.length", kind),
        media_description.payloads.len() as i32,
    );

    for (i, (uri, id)) in media_description.extensions.iter().enumerate() {
        properties.set_int(&format!("{}.ext.{}.id", kind, i), *id as i32);
        properties.set_string(&format!("{}.ext.{}.uri", kind, i), uri);
    }

    properties.set_int(
        &format!("{}.ext.length", kind),
        media_description.extensions.len() as i32,
    );
}

fn get_rtp_properties_from_sdp(sdp: &UnifiedBundleSession) -> Properties {
    let mut properties = Properties::new();

    let first_audio_media = sdp.media_descriptions.iter().find(|md| md.kind == MediaType::Audio);

    if let Some(media_description) = first_audio_media {
        add_rtp_properties_from_media_description(&mut properties, media_description);
    }

    let first_video_media = sdp.media_descriptions.iter().find(|md| md.kind == MediaType::Video);

    if let Some(media_description) = first_video_media {
        add_rtp_properties_from_media_description(&mut properties, media_description);
    }

    properties
}

/// Filters the codecs, rtcp feedbacks, and extensions in the SDP according to
/// the media-server capabilities.
fn filter_answer_to_capabilities(sdp: &mut UnifiedBundleSession) {
    for media_description in &mut sdp.media_descriptions {
        let kind = media_description.kind.clone();

        media_description.payloads.retain(|payload| match kind {
            MediaType::Audio => match payload.name {
                RtpCodecName::Opus => true,
                RtpCodecName::Pcmu => true,
                RtpCodecName::Pcma => true,
                _ => false,
            },
            MediaType::Video => match payload.name {
                RtpCodecName::Vp8 => true,
                RtpCodecName::Vp9 => true,
                RtpCodecName::H264 => match payload.parameters.get("packetization-mode") {
                    Some(mode) => mode == "1",
                    None => false,
                },
                _ => false,
            },
            _ => false,
        });

        for payload in &mut media_description.payloads {
            payload.supported_feedback.retain(|id, param| match kind {
                MediaType::Video => match (id.as_str(), param.as_deref()) {
                    ("goog-remb", None) => true,
                    ("transport-cc", None) => true,
                    ("ccm", Some("fir")) => true,
                    ("nack", None) => true,
                    ("nack", Some("pli")) => true,
                    _ => false,
                },
                _ => false,
            });
        }

        media_description.extensions.retain(|uri, _id| match kind {
            MediaType::Audio => match uri.as_str() {
                "urn:ietf:params:rtp-hdrext:ssrc-audio-level" => true,
                "urn:ietf:params:rtp-hdrext:sdes:mid" => true,
                "urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id" => true,
                "http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time" => true,
                _ => false,
            },
            MediaType::Video => match uri.as_str() {
                "urn:3gpp:video-orientation" => true,
                "http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01" => true,
                "urn:ietf:params:rtp-hdrext:sdes:mid" => true,
                "urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id" => true,
                "urn:ietf:params:rtp-hdrext:sdes:repaired-rtp-stream-id" => true,
                "http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time" => true,
                _ => false,
            },
            _ => false,
        });
    }
}

#[allow(dead_code)]
struct ActiveSession {
    transport: RtpBundleTransport,
    connection: RtpBundleTransportConnection,
    incoming_source_groups: Vec<RtpIncomingSourceGroup>,
}

async fn handle_offer(
    opts: Arc<Opts>,
    websocket: &mut warp::ws::WebSocket,
    offer: &UnifiedBundleSession,
) -> Result<ActiveSession, Box<dyn Error>> {
    // TODO: We want to implement something along the lines of the
    //       media-server-node manual signalling example in here.
    //       https://github.com/medooze/media-server-node/blob/master/manual.md

    // TODO: We haven't implemented the higher-level Endpoint / Transport APIs in media-server yet,
    //       so we're just gonna use the raw native API to get something running here,
    //       and use it to guide implementation of those APIs later.

    // TODO: We shouldn't be creating one RtpBundleTransport (Endpoint) per connection.
    let transport = RtpBundleTransport::new(None)?;

    // This will generate a new ice ufrag/pwd,
    // we need to add our ICE candidates and DTLS fingerprint.
    let mut answer = offer.answer();

    filter_answer_to_capabilities(&mut answer);

    answer.ice_lite = true;

    answer.candidates.push(Candidate {
        foundation: "1".to_owned(),
        component: 1,
        transport: IceTransportType::Udp,
        priority: (2u32.pow(24) * 126) + (2u32.pow(8) * (65535 - 1)) + 255,
        address: opts.public_ip.to_string(),
        port: transport.get_local_port(),
        kind: IceCandidateType::Host,
        rel_addr: None,
        rel_port: None,
        unknown: Default::default(),
        tcp_type: None,
    });

    // TODO: media_server::get_certificate_fingerprint should probably return these in the right type already
    let our_fingerprint = media_server::get_certificate_fingerprint(DtlsConnectionHash::Sha256)?;
    answer.fingerprints.append(
        FingerprintHashFunction::Sha256,
        CertificateFingerprint::from_str(&our_fingerprint)?,
    );

    let offer_fingerprint = offer
        .fingerprints
        .get(&FingerprintHashFunction::Sha256)
        .ok_or("sha-256 dtls fingerprint missing from offer")?;

    let properties = Properties::new();
    properties.set_string("ice.localUsername", &answer.ice_ufrag);
    properties.set_string("ice.localPassword", &answer.ice_pwd);
    properties.set_string("ice.remoteUsername", &offer.ice_ufrag);
    properties.set_string("ice.remotePassword", &offer.ice_pwd);
    properties.set_string("dtls.setup", offer.setup_role.as_ref());
    properties.set_string("dtls.hash", "SHA-256");
    properties.set_string("dtls.fingerprint", &offer_fingerprint.to_string());
    properties.set_bool("disableSTUNKeepAlive", false);
    properties.set_string("srtpProtectionProfiles", "");

    let username = answer.ice_ufrag.clone() + ":" + &offer.ice_ufrag;
    let connection = transport.add_ice_transport(username.as_str(), &properties)?;

    let remote_properties = get_rtp_properties_from_sdp(offer);
    connection.set_remote_properties(&remote_properties);

    let local_properties = get_rtp_properties_from_sdp(&answer);
    connection.set_local_properties(&local_properties);

    let mut incoming_source_groups = Vec::new();

    // TODO: We've got a weird bug here where media-server isn't matching up the RTX
    //       packets with an encoding - both the MID and RID headers seems to be missing.
    //       Need to test with media-server-node.
    //       This still happens with media-server-node, the issue appears to be Chrome
    //       not sending MID/RID in RTCP packets when doing simulcast and the matching
    //       encoding is not currently active. Doesn't look like there is anything to do
    //       and it recovers happily once all of the encodings become active.

    for media_description in &offer.media_descriptions {
        let frame_type = match media_description.kind {
            MediaType::Audio => MediaFrameType::Audio,
            MediaType::Video => MediaFrameType::Video,
            _ => continue,
        };

        for encoding in &media_description.encodings {
            let incoming_source_group = match encoding {
                RtpEncoding::Rid { rid, .. } => connection.add_incoming_source_group(
                    frame_type,
                    Some(&media_description.mid.0),
                    Some(&rid.0),
                    None,
                    None,
                )?,
                RtpEncoding::SendingSsrc { ssrc, rtx_ssrc, .. } => connection.add_incoming_source_group(
                    frame_type,
                    Some(&media_description.mid.0),
                    None,
                    Some(ssrc.0),
                    rtx_ssrc.map(|s| s.0),
                )?,
            };

            incoming_source_groups.push(incoming_source_group);
        }
    }

    // TODO: Mirror back the tracks?

    send_message(websocket, &S2CMessage::Answer { sdp: answer }).await?;

    Ok(ActiveSession {
        transport,
        connection,
        incoming_source_groups,
    })
}

async fn on_websocket_upgrade(opts: Arc<Opts>, mut websocket: warp::ws::WebSocket) {
    // Stores the media-server objects for the current websocket
    let mut session = None;

    while let Ok(Some(message)) = websocket.try_next().await {
        if message.is_close() {
            log::info!("client closed websocket");
            let _ = websocket.close().await;
            return;
        }

        let text = match message.to_str() {
            Ok(text) => text,
            Err(()) => {
                log::warn!("unexpected message type in websocket: {:?}", message);
                return;
            }
        };

        log::info!("message: {}", text);

        let parsed: C2SMessage = match serde_json::from_str(text) {
            Ok(parsed) => parsed,
            Err(e) => {
                log::warn!("failed to parse websocket message: {}", e);
                return;
            }
        };

        log::info!("parsed: {:?}", parsed);

        match parsed {
            C2SMessage::Offer { sdp } => {
                match handle_offer(opts.clone(), &mut websocket, &sdp).await {
                    Ok(new_session) => session.replace(new_session),
                    Err(e) => {
                        log::warn!("failed to handle offer: {}", e);
                        return;
                    }
                };
            }
        };
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let opts = Arc::new(Opts::from_args());

    let opts_filter_clone = opts.clone();
    let opts_filter = warp::any().map(move || opts_filter_clone.clone());

    media_server::library_init(LoggingLevel::Debug).unwrap();

    if opts.port_range.is_some() {
        media_server::set_port_range(opts.port_range).unwrap();
    }

    let websocket = warp::get()
        .and(warp::path::path("ws"))
        .and(warp::path::end())
        .and(warp::ws())
        .and(opts_filter)
        .map(|ws: warp::ws::Ws, opts: Arc<Opts>| ws.on_upgrade(|w| on_websocket_upgrade(opts, w)));

    let index = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::html(include_str!("../resources/index.html")));

    let adapter = warp::get()
        .and(warp::path::path("adapter.js"))
        .and(warp::path::end())
        .map(|| {
            let adapter = include_str!("../resources/adapter.js");
            warp::reply::with_header(adapter, "Content-Type", "application/javascript")
        });

    let favicon = warp::get()
        .and(warp::path::path("favicon.ico"))
        .and(warp::path::end())
        .map(|| {
            let favicon = include_bytes!("../resources/favicon.ico");
            warp::reply::with_header(favicon.as_ref(), "Content-Type", "image/vnd.microsoft.icon")
        });

    let routes = websocket
        .or(index)
        .or(adapter)
        .or(favicon)
        .with(warp::log("media_server_demo::http"));

    warp::serve(routes).run(opts.listen).await;
}
