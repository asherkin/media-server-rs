//! This module implements the SDP parts of the JSEP specification.
//!
//! <https://rtcweb-wg.github.io/jsep/#rfc.section.5>

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use ordered_multimap::ListOrderedMultimap;
use rand::Rng;

use crate::attributes::{
    Candidate, ExtensionMap, ExtensionMapAllowMixed, Fingerprint, FormatParameters, Group, IceLite, IceOptions, IcePwd,
    IceUfrag, Inactive, Mid, ReceiveOnly, Rid, Rtcp, RtcpFeedback, RtcpMux, RtcpMuxOnly, RtcpReducedSize, RtpMap,
    SendOnly, SendReceive, Setup, SsrcAttribute, SsrcGroup,
};
use crate::enums::{
    AddressType, BandwidthType, FingerprintHashFunction, GroupSemantics, IceOption, MediaType, NetworkType,
    RidDirection, RtpCodecName, SetupRole, SsrcGroupSemantics, TransportProtocol,
};
use crate::types::{CertificateFingerprint, PayloadType, Ssrc};
use crate::{sdp, types, AttributeMap};

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

    pub allow_mixed_extension_maps: bool,

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
            setup_role: SetupRole::ActivePassive,
            allow_mixed_extension_maps: true,
            media_descriptions: Vec::new(),
        }
    }

    pub fn answer(&self) -> UnifiedBundleSession {
        use rand::distributions::Alphanumeric;
        let mut rng = rand::thread_rng();

        UnifiedBundleSession {
            id: rng.gen_range(0, 9_223_372_036_854_775_807),
            version: 1,
            ice_lite: false,
            ice_ufrag: rng.sample_iter(Alphanumeric).take(8).collect(),
            ice_pwd: rng.sample_iter(Alphanumeric).take(24).collect(),
            ice_options: HashSet::new(),
            candidates: Vec::new(),
            fingerprints: ListOrderedMultimap::new(),
            setup_role: self.setup_role.reverse(),
            allow_mixed_extension_maps: self.allow_mixed_extension_maps,
            media_descriptions: self.media_descriptions.iter().map(|md| md.answer()).collect(),
        }
    }

    /// <https://rtcweb-wg.github.io/jsep/#rfc.section.5.8>
    pub fn from_sdp(sdp: &sdp::Session) -> Result<Self, String> {
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

        let allow_mixed_extension_maps = first_media_description
            .attributes
            .get::<ExtensionMapAllowMixed>()
            .or_else(|| sdp.attributes.get())
            .is_some();

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
            allow_mixed_extension_maps,
            media_descriptions,
        };

        Ok(session)
    }

    pub fn to_sdp(&self) -> sdp::Session {
        let mut attributes = AttributeMap::new();

        if self.ice_lite {
            attributes.append(IceLite);
        }

        if !self.media_descriptions.is_empty() {
            attributes.append(Group {
                semantics: GroupSemantics::Bundle,
                mids: self.media_descriptions.iter().map(|md| md.mid.clone()).collect(),
            });

            // TODO: msid-semantic ?
        }

        if self.allow_mixed_extension_maps {
            attributes.append(ExtensionMapAllowMixed);
        }

        let media_descriptions = self.media_descriptions.iter().map(|md| md.to_sdp(self)).collect();

        sdp::Session {
            origin: sdp::Origin {
                username: None,
                session_id: self.id,
                session_version: self.version,
                network_type: NetworkType::Internet,
                address_type: AddressType::Ip4,
                unicast_address: "127.0.0.1".to_owned(),
            },
            name: None,
            information: None,
            uri: None,
            email_address: None,
            phone_number: None,
            connection: None,
            bandwidths: HashMap::new(),
            times: vec![sdp::Time {
                start: 0,
                stop: 0,
                repeat_times: Vec::new(),
                time_zone_adjustments: Vec::new(),
            }],
            encryption_key: None,
            attributes,
            media_descriptions,
        }
    }
}

impl FromStr for UnifiedBundleSession {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let session = sdp::Session::from_str(s)?;
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

impl MediaDirection {
    pub fn reverse(&self) -> MediaDirection {
        match self {
            MediaDirection::SendOnly => MediaDirection::ReceiveOnly,
            MediaDirection::ReceiveOnly => MediaDirection::SendOnly,
            MediaDirection::SendReceive => MediaDirection::SendReceive,
            MediaDirection::Inactive => MediaDirection::Inactive,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RtpPayload {
    pub payload_type: PayloadType,
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
    SendingSsrc {
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

    pub payloads: Vec<RtpPayload>,

    pub direction: MediaDirection,

    pub encodings: Vec<RtpEncoding>,

    pub extensions: HashMap<String, u16>,

    pub rtcp_mux: bool,
    pub rtcp_mux_only: bool,
    pub rtcp_reduced_size: bool,
    // msid, imageattr, simulcast
}

impl RtpMediaDescription {
    pub fn answer(&self) -> RtpMediaDescription {
        let encodings = self
            .encodings
            .iter()
            .filter_map(|encoding| match encoding {
                RtpEncoding::Rid { rid, direction } => Some(RtpEncoding::Rid {
                    rid: rid.clone(),
                    direction: direction.reverse(),
                }),
                RtpEncoding::SendingSsrc { .. } => None,
            })
            .collect();

        RtpMediaDescription {
            kind: self.kind.clone(),
            port: self.port,
            protocol: self.protocol.clone(),
            bandwidths: HashMap::new(),
            mid: self.mid.clone(),
            payloads: self.payloads.clone(),
            direction: self.direction.reverse(),
            encodings,
            extensions: self.extensions.clone(),
            rtcp_mux: self.rtcp_mux,
            rtcp_mux_only: self.rtcp_mux_only,
            rtcp_reduced_size: self.rtcp_reduced_size,
        }
    }

    /// <https://rtcweb-wg.github.io/jsep/#rfc.section.5.8>
    pub fn from_sdp(sdp: &sdp::MediaDescription) -> Result<Self, String> {
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

        let payloads: Vec<_> = sdp
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
                    payload_type: fmt,
                    name: map.name.clone(),
                    clock: map.clock,
                    channels: map.channels,
                    parameters,
                    supported_feedback,
                    rtx_payload_type,
                };

                Some(payload)
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

        let rtcp_mux = sdp.attributes.get::<RtcpMux>().is_some();
        let rtcp_mux_only = sdp.attributes.get::<RtcpMuxOnly>().is_some();
        let rtcp_reduced_size = sdp.attributes.get::<RtcpReducedSize>().is_some();

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
            let ssrc_encoding = RtpEncoding::SendingSsrc { cname, ssrc, rtx_ssrc };

            encodings.push(ssrc_encoding);
        }

        let extensions = sdp
            .attributes
            .get_vec::<ExtensionMap>()
            .into_iter()
            .filter_map(|map| {
                if map.direction.is_some() {
                    // Directional extensions are not supported.
                    // TODO: Firefox uses them.
                    return None;
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
            rtcp_mux,
            rtcp_mux_only,
            rtcp_reduced_size,
        };

        Ok(media_description)
    }

    pub fn to_sdp(&self, session: &UnifiedBundleSession) -> sdp::MediaDescription {
        let mut attributes = AttributeMap::new();

        attributes.append(Rtcp {
            port: 9,
            network_type: Some(NetworkType::Internet),
            address_type: Some(AddressType::Ip4),
            connection_address: Some("0.0.0.0".to_owned()),
        });

        attributes.append(IceUfrag(session.ice_ufrag.clone()));
        attributes.append(IcePwd(session.ice_pwd.clone()));

        if !session.ice_options.is_empty() {
            attributes.append(IceOptions(session.ice_options.clone()));
        }

        for candidate in &session.candidates {
            attributes.append(candidate.clone());
        }

        for (hash_function, fingerprint) in &session.fingerprints {
            attributes.append(Fingerprint {
                hash_function: hash_function.clone(),
                fingerprint: fingerprint.clone(),
            });
        }

        attributes.append(Setup(session.setup_role.clone()));

        attributes.append(Mid(self.mid.clone()));

        for (extension, &id) in &self.extensions {
            attributes.append(ExtensionMap {
                id,
                direction: None,
                extension: extension.clone(),
                attributes: Vec::new(),
            });
        }

        match self.direction {
            MediaDirection::SendOnly => attributes.append(SendOnly),
            MediaDirection::ReceiveOnly => attributes.append(ReceiveOnly),
            MediaDirection::SendReceive => attributes.append(SendReceive),
            MediaDirection::Inactive => attributes.append(Inactive),
        }

        // TODO: msid ?

        if self.rtcp_mux {
            attributes.append(RtcpMux);
        }

        if self.rtcp_mux_only {
            attributes.append(RtcpMuxOnly);
        }

        if self.rtcp_reduced_size {
            attributes.append(RtcpReducedSize);
        }

        let mut formats = Vec::new();

        for payload in &self.payloads {
            formats.push(payload.payload_type.to_string());

            attributes.append(RtpMap {
                payload: payload.payload_type,
                name: payload.name.clone(),
                clock: payload.clock,
                channels: payload.channels,
            });

            for (feedback_id, feedback_param) in &payload.supported_feedback {
                attributes.append(RtcpFeedback {
                    payload: Some(payload.payload_type),
                    id: feedback_id.clone(),
                    param: feedback_param.clone(),
                });
            }

            if !payload.parameters.is_empty() {
                let parameters = payload
                    .parameters
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(";");

                attributes.append(FormatParameters {
                    payload: payload.payload_type,
                    parameters,
                });
            }

            if let Some(rtx_payload_type) = payload.rtx_payload_type {
                formats.push(rtx_payload_type.to_string());

                attributes.append(RtpMap {
                    payload: rtx_payload_type,
                    name: RtpCodecName::Rtx,
                    clock: payload.clock,
                    channels: payload.channels,
                });

                attributes.append(FormatParameters {
                    payload: rtx_payload_type,
                    parameters: format!("apt={}", payload.payload_type),
                });
            }
        }

        for encoding in &self.encodings {
            match encoding {
                RtpEncoding::Rid { rid, direction } => {
                    attributes.append(Rid {
                        rid: rid.clone(),
                        direction: direction.clone(),
                        restrictions: None,
                    });
                }
                RtpEncoding::SendingSsrc { cname, ssrc, rtx_ssrc } => {
                    attributes.append(SsrcAttribute {
                        ssrc: *ssrc,
                        name: "cname".to_owned(),
                        value: Some(cname.clone()),
                    });

                    // TODO: We should add the msid / label / mslabel attributes at some point.

                    if let Some(rtx_ssrc) = rtx_ssrc {
                        attributes.append(SsrcAttribute {
                            ssrc: *rtx_ssrc,
                            name: "cname".to_owned(),
                            value: Some(cname.clone()),
                        });

                        // TODO: Firefox doesn't appear to include a ssrc-group?
                        attributes.append(SsrcGroup {
                            semantics: SsrcGroupSemantics::FlowIdentification,
                            ssrcs: vec![*ssrc, *rtx_ssrc],
                        });
                    }
                }
            }
        }

        let mut simulcast_value = String::new();

        let send_rid_encodings: Vec<_> = self
            .encodings
            .iter()
            .filter_map(|e| match e {
                RtpEncoding::Rid {
                    rid,
                    direction: RidDirection::Send,
                } => Some(rid.0.clone()),
                _ => None,
            })
            .collect();

        if !send_rid_encodings.is_empty() {
            simulcast_value += &format!("send {}", send_rid_encodings.join(";"))
        }

        let recv_rid_encodings: Vec<_> = self
            .encodings
            .iter()
            .filter_map(|e| match e {
                RtpEncoding::Rid {
                    rid,
                    direction: RidDirection::Receive,
                } => Some(rid.0.clone()),
                _ => None,
            })
            .collect();

        if !recv_rid_encodings.is_empty() {
            if !send_rid_encodings.is_empty() {
                simulcast_value += " ";
            }

            simulcast_value += &format!("recv {}", recv_rid_encodings.join(";"))
        }

        if !simulcast_value.is_empty() {
            // TODO: We haven't implemented a type for this attribute yet,
            //       as the full parsing of it is fairly complex.
            attributes.append_unknown("simulcast", Some(simulcast_value)).unwrap();
        }

        sdp::MediaDescription {
            kind: self.kind.clone(),
            port: 9,
            num_ports: None,
            protocol: self.protocol.clone(),
            formats,
            title: None,
            connection: Some(sdp::Connection {
                network_type: NetworkType::Internet,
                address_type: AddressType::Ip4,
                connection_address: "0.0.0.0".to_owned(),
            }),
            bandwidths: self.bandwidths.clone(),
            encryption_key: None,
            attributes,
        }
    }
}
