//! This module implements the SDP parts of the JSEP specification.
//!
//! https://rtcweb-wg.github.io/jsep/#rfc.section.5

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use ordered_multimap::ListOrderedMultimap;
use rand::Rng;

use crate::attributes::{
    Candidate, ExtensionMap, Fingerprint, FormatParameters, IceLite, IceOptions, IcePwd, IceUfrag, Inactive, Mid,
    ReceiveOnly, Rid, RtcpFeedback, RtpMap, SendOnly, SendReceive, Setup, SsrcAttribute, SsrcGroup,
};
use crate::enums::{
    BandwidthType, FingerprintHashFunction, IceOption, MediaType, RidDirection, RtpCodecName, SetupRole,
    SsrcGroupSemantics, TransportProtocol,
};
use crate::types;
use crate::types::{CertificateFingerprint, PayloadType, Ssrc};

mod tests;

/// Simplified SDP representation for a unified-plan WebRTC session with a single bundled transport.
///
/// A lot of functionality is unable to be represented here, but it should have enough to negotiate
/// a RTP-only multi-track session with modern versions of the major browsers.
#[derive(Debug, Clone)]
pub struct UnifiedBundleSession {
    pub id: u64,
    pub version: u64,

    pub ice_lite: bool,
    pub ice_ufrag: String,
    pub ice_pwd: String,
    pub ice_options: HashSet<IceOption>,
    pub candidates: Vec<Candidate>,

    pub fingerprints: ListOrderedMultimap<FingerprintHashFunction, CertificateFingerprint>,
    pub setup_role: SetupRole,

    // TODO: Support non-RTP media
    pub media_descriptions: Vec<RtpMediaDescription>,
}

impl UnifiedBundleSession {
    // TODO: This probably needs a builder class.
    pub fn new() -> UnifiedBundleSession {
        use rand::distributions::Alphanumeric;
        let mut rng = rand::thread_rng();

        UnifiedBundleSession {
            id: rng.gen_range(0, 9_223_372_036_854_775_807),
            version: 1,
            ice_lite: true,
            ice_ufrag: rng.sample_iter(Alphanumeric).take(8).collect(),
            ice_pwd: rng.sample_iter(Alphanumeric).take(24).collect(),
            ice_options: HashSet::new(),
            candidates: Vec::new(),
            fingerprints: ListOrderedMultimap::new(),
            setup_role: SetupRole::Passive,
            media_descriptions: Vec::new(),
        }
    }

    pub fn answer(&self) -> UnifiedBundleSession {
        todo!()
    }

    /// https://rtcweb-wg.github.io/jsep/#rfc.section.5.8
    pub fn from_sdp(sdp: &crate::sdp::Session) -> Result<Self, String> {
        let ice_lite = sdp.attributes.get::<IceLite>().is_some();

        // Get ICE and DTLS info from the first m-line.
        // TODO: We're lazy and just assume max-bundle if this struct is being used.
        // TODO: WebRTC can generate an offer with 0 m-lines as part of a perfect
        //       negotiation strategy, we should handle it although can do nothing useful.
        let first_media_description = sdp
            .media_descriptions
            .first()
            .ok_or("at least one m-line is required")?;

        let ice_ufrag = first_media_description
            .attributes
            .get::<IceUfrag>()
            .or_else(|| sdp.attributes.get())
            .ok_or("ice-ufrag is required")?
            .0
            .clone();

        let ice_pwd = first_media_description
            .attributes
            .get::<IcePwd>()
            .or_else(|| sdp.attributes.get())
            .ok_or("ice-pwd is required")?
            .0
            .clone();

        let ice_options = first_media_description
            .attributes
            .get::<IceOptions>()
            .or_else(|| sdp.attributes.get())
            .map_or_else(HashSet::new, |a| a.0.clone());

        let candidates = first_media_description
            .attributes
            .get_vec::<Candidate>()
            .into_iter()
            .cloned()
            .collect();

        let fingerprints = first_media_description
            .attributes
            .get_vec::<Fingerprint>()
            .into_iter()
            .chain(sdp.attributes.get_vec())
            .map(|a| (a.hash_function.clone(), a.fingerprint.clone()))
            .collect();

        let setup_role = first_media_description
            .attributes
            .get::<Setup>()
            .or_else(|| sdp.attributes.get())
            .map(|a| a.0.clone())
            .unwrap_or(SetupRole::ActivePassive);

        let media_descriptions = sdp
            .media_descriptions
            .iter()
            .map(RtpMediaDescription::from_sdp)
            .collect::<Result<Vec<_>, _>>()?;

        let session = UnifiedBundleSession {
            id: sdp.origin.session_id,
            version: sdp.origin.session_version,
            ice_lite,
            ice_ufrag,
            ice_pwd,
            ice_options,
            candidates,
            fingerprints,
            setup_role,
            media_descriptions,
        };

        Ok(session)
    }

    pub fn to_sdp(&self) -> crate::sdp::Session {
        todo!()
    }
}

impl FromStr for UnifiedBundleSession {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let session = crate::sdp::Session::from_str(s)?;
        let session = Self::from_sdp(&session)?;
        Ok(session)
    }
}

impl std::fmt::Display for UnifiedBundleSession {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.to_sdp().fmt(f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MediaDirection {
    SendOnly,
    ReceiveOnly,
    SendReceive,
    Inactive,
}

#[derive(Debug, Clone)]
pub struct RtpPayload {
    pub name: RtpCodecName,
    pub clock: u32,
    pub channels: Option<u8>,
    pub parameters: HashMap<String, String>,
    pub supported_feedback: ListOrderedMultimap<String, Option<String>>,
    pub rtx_payload_type: Option<PayloadType>,
}

#[derive(Debug, Clone)]
pub enum RtpEncoding {
    Rid {
        rid: types::Rid,
        direction: RidDirection,
    },
    Ssrc {
        cname: String,
        ssrc: Ssrc,
        rtx_ssrc: Option<Ssrc>,
    },
}

#[derive(Debug, Clone)]
pub struct RtpMediaDescription {
    pub kind: MediaType,
    pub port: u16,
    pub protocol: TransportProtocol,

    pub bandwidths: HashMap<BandwidthType, u64>,

    pub mid: types::Mid,

    pub payloads: HashMap<PayloadType, RtpPayload>,

    pub direction: MediaDirection,

    // TODO: Add `encodings` which will scoop up rid/ssrc(-group) info
    pub encodings: Vec<RtpEncoding>,

    // ssrc attributes

    // TODO: Parse these at some level.
    // pub ssrc_attributes: HashMap<u32, HashMap<String, Option<String>>>,

    // First SSRC of FID / FEC-FR is primary stream, 2nd is the RTX / FEC stream
    // TODO: For those groups we want fast lookup by SSRC
    //       Actually maybe there can only be one group per m-line?
    //       plan-b = multiple ssrc groups per m-line
    //       unified = single set of ssrc groups per m-line
    // pub ssrc_groups: Vec<SsrcGroup>,
    pub extensions: HashMap<String, u16>,
    // rtcp-fb, rtcp-mux?, rtcp-mux-only?, rtcp-rsize?
    // msid, imageattr, rid, simulcast
}

impl RtpMediaDescription {
    /// https://rtcweb-wg.github.io/jsep/#rfc.section.5.8
    pub fn from_sdp(sdp: &crate::sdp::MediaDescription) -> Result<Self, String> {
        let mid = sdp.attributes.get::<Mid>().ok_or("mid is required")?.0.clone();

        let rtp_maps: HashMap<_, _> = sdp
            .attributes
            .get_vec::<RtpMap>()
            .into_iter()
            .map(|a| (a.payload, a))
            .collect();

        let format_parameters: HashMap<_, _> = sdp
            .attributes
            .get_vec::<FormatParameters>()
            .into_iter()
            .map(|a| {
                let parameters: HashMap<_, _> = a
                    .parameters
                    .split(';')
                    .filter_map(|parameter| {
                        // A few codecs don't use the recommended key=value form,
                        // but we don't care about those so just ignore any without a '='.
                        let (k, v) = parameter.split_at(parameter.find('=')?);
                        Some((k.to_owned(), v[1..].to_owned()))
                    })
                    .collect();

                (a.payload, parameters)
            })
            .collect();

        let rtx_payload_map: HashMap<_, _> = rtp_maps
            .iter()
            .filter_map(|(fmt, map)| {
                if map.name != RtpCodecName::Rtx {
                    return None;
                }

                let attributes = format_parameters.get(fmt)?;
                let apt = attributes.get("apt")?;
                let apt = PayloadType::from_str(apt).ok()?;

                Some((apt, *fmt))
            })
            .collect();

        let supported_feedback: ListOrderedMultimap<_, _> = sdp
            .attributes
            .get_vec::<RtcpFeedback>()
            .into_iter()
            .filter_map(|a| Some((a.payload?, (&a.id, &a.param))))
            .collect();

        let payloads: HashMap<_, _> = sdp
            .formats
            .iter()
            .filter_map(|fmt| PayloadType::from_str(fmt).ok())
            .filter_map(|fmt| {
                let map = rtp_maps.get(&fmt)?;

                // Filter out known non-media types.
                match map.name {
                    RtpCodecName::Rtx => return None,
                    RtpCodecName::Red => return None,
                    RtpCodecName::UlpFec => return None,
                    RtpCodecName::FlexFec => return None,
                    _ => (),
                }

                let parameters = format_parameters.get(&fmt).map_or_else(HashMap::new, Clone::clone);

                let supported_feedback = supported_feedback
                    .get_all(&fmt)
                    .map(|&(id, param)| (id.to_owned(), param.to_owned()))
                    .collect();

                let rtx_payload_type = rtx_payload_map.get(&fmt).cloned();

                let payload = RtpPayload {
                    name: map.name.clone(),
                    clock: map.clock,
                    channels: map.channels,
                    parameters,
                    supported_feedback,
                    rtx_payload_type,
                };

                Some((fmt, payload))
            })
            .collect();

        let has_sendonly = sdp.attributes.get::<SendOnly>().is_some();
        let has_recvonly = sdp.attributes.get::<ReceiveOnly>().is_some();
        let has_sendrecv = sdp.attributes.get::<SendReceive>().is_some();
        let has_inactive = sdp.attributes.get::<Inactive>().is_some();
        let direction = match (has_sendonly, has_recvonly, has_sendrecv, has_inactive) {
            (true, false, false, false) => MediaDirection::SendOnly,
            (false, true, false, false) => MediaDirection::ReceiveOnly,
            (false, false, _, false) => MediaDirection::SendReceive,
            (false, false, false, true) => MediaDirection::Inactive,
            _ => return Err("Multiple direction attributes found in m-line".to_owned()),
        };

        let mut encodings = Vec::new();

        // TODO: Right now we ignore the simulcast attribute and just assume all RIDs are simulcast.

        let rid_encodings = sdp.attributes.get_vec::<Rid>().into_iter().filter_map(|attribute| {
            if attribute.restrictions.is_some() {
                // TODO: Restricted RIDs can't currently be represented.
                return None;
            }

            let rid_encoding = RtpEncoding::Rid {
                rid: attribute.rid.clone(),
                direction: attribute.direction.clone(),
            };

            Some(rid_encoding)
        });

        encodings.extend(rid_encodings);

        // TODO: Only look for ssrc-based encodings if no rid-based encodings specified?

        let ssrc_groups: HashMap<_, _> = sdp
            .attributes
            .get_vec::<SsrcGroup>()
            .into_iter()
            .map(|group| (&group.semantics, &group.ssrcs))
            .collect();

        let ssrc_attributes: HashMap<_, _> = sdp
            .attributes
            .get_vec::<SsrcAttribute>()
            .into_iter()
            .map(|attribute| (attribute.ssrc, attribute))
            .collect::<ListOrderedMultimap<_, _>>()
            .drain_pairs()
            .map(|(ssrc, attributes)| {
                let attributes: HashMap<_, _> = attributes
                    .map(|attribute| (attribute.name.to_ascii_lowercase(), attribute.value.clone()))
                    .collect();

                (ssrc, attributes)
            })
            .collect();

        let fid_group = ssrc_groups.get(&SsrcGroupSemantics::FlowIdentification);
        let (ssrc, rtx_ssrc) = match fid_group {
            Some(group) => (group.get(0).cloned(), group.get(1).cloned()),
            None => (ssrc_attributes.keys().next().cloned(), None),
        };

        let cname = ssrc.and_then(|ssrc| ssrc_attributes.get(&ssrc)?.get("cname")?.clone());

        if let (Some(ssrc), Some(cname)) = (ssrc, cname) {
            let ssrc_encoding = RtpEncoding::Ssrc { cname, ssrc, rtx_ssrc };

            encodings.push(ssrc_encoding);
        }

        let extensions = sdp
            .attributes
            .get_vec::<ExtensionMap>()
            .into_iter()
            .filter_map(|map| {
                if map.direction.is_some() {
                    // Directional extensions are not supported.
                    // TODO: Looks like Firefox might use them, just ignore the direction for now.
                    // return None;
                }

                if !map.attributes.is_empty() {
                    // Extensions with attributes are not supported.
                    return None;
                }

                Some((map.extension.clone(), map.id))
            })
            .collect();

        let media_description = RtpMediaDescription {
            kind: sdp.kind.clone(),
            port: sdp.port,
            protocol: sdp.protocol.clone(),
            bandwidths: sdp.bandwidths.clone(),
            mid,
            payloads,
            direction,
            encodings,
            extensions,
        };

        Ok(media_description)
    }

    pub fn to_sdp(&self) -> crate::sdp::Session {
        todo!()
    }
}
