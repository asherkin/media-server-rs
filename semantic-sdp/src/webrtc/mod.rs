//! This module implements the SDP parts of the JSEP specification.
//!
//! https://rtcweb-wg.github.io/jsep/#rfc.section.5

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use ordered_multimap::ListOrderedMultimap;
use rand::Rng;

use crate::attributes::{
    Candidate, EndOfCandidates, ExtensionMap, Fingerprint, FormatParameters, Group, IceLite, IceOptions, IcePwd,
    IceUfrag, Inactive, MaxPacketTime, Mid, PacketTime, ReceiveOnly, RtpMap, SendOnly, SendReceive, Setup,
};
use crate::enums::{BandwidthType, FingerprintHashFunction, IceOption, MediaType, SetupRole, TransportProtocol};

mod tests;

// TODO: Consider adding a set of newtypes for the common ones we don't want mixed up,
//       e.g. fingerprints, SSRCs, MIDs, RIDs, RTP PTs

// TODO: While higher level than the raw SDP, this is still quite low-level.
//       We'd like this module to be quite a high-level interface, but maybe
//       that needs to be built as yet another level of abstraction on top.
// TODO: Alternatively, JS semantic-sdp clearly works quite well in the real
//       world. We could ignore the spec behaviour here and use the parsing
//       strategy from there - which is a lot simpler and has a nicer interface,
//       at the cost of embedding non-spec assumptions about the SDP format
//       and restricting the functionality supported.
#[derive(Debug)]
pub struct Session {
    pub id: u64,
    pub version: u64,
    pub bandwidths: HashMap<BandwidthType, u64>,

    pub groups: Vec<Group>,

    pub ice_lite: bool,
    pub ice_ufrag: Option<String>,
    pub ice_pwd: Option<String>,
    pub ice_options: HashSet<IceOption>,

    pub fingerprints: ListOrderedMultimap<FingerprintHashFunction, Vec<u8>>,
    pub setup_role: Option<SetupRole>,

    pub extensions: Vec<ExtensionMap>,

    // TODO: Support non-RTP media
    pub media_descriptions: Vec<RtpMediaDescription>,
}

impl Session {
    // TODO: This probably needs a builder class.
    pub fn new() -> Session {
        use rand::distributions::Alphanumeric;
        let mut rng = rand::thread_rng();

        Session {
            id: rng.gen_range(0, 9_223_372_036_854_775_807),
            version: 1,
            bandwidths: HashMap::new(),
            groups: Vec::new(),
            ice_lite: false,
            ice_ufrag: Some(rng.sample_iter(Alphanumeric).take(8).collect()),
            ice_pwd: Some(rng.sample_iter(Alphanumeric).take(24).collect()),
            ice_options: HashSet::new(),
            fingerprints: ListOrderedMultimap::new(),
            setup_role: None,
            extensions: Vec::new(),
            media_descriptions: Vec::new(),
        }
    }

    pub fn answer(&self) -> Session {
        todo!()
    }

    /// https://rtcweb-wg.github.io/jsep/#rfc.section.5.8
    pub fn from_sdp(sdp: &crate::sdp::Session) -> Result<Self, String> {
        let groups = sdp.attributes.get_vec::<Group>().into_iter().cloned().collect();

        let ice_lite = sdp.attributes.get::<IceLite>().is_some();
        let ice_ufrag = sdp.attributes.get::<IceUfrag>().map(|a| a.0.clone());
        let ice_pwd = sdp.attributes.get::<IcePwd>().map(|a| a.0.clone());
        let ice_options = sdp
            .attributes
            .get::<IceOptions>()
            .map_or_else(HashSet::new, |a| a.0.clone());

        let fingerprints = sdp
            .attributes
            .get_vec::<Fingerprint>()
            .into_iter()
            .map(|a| (a.hash_function.clone(), a.fingerprint.clone()))
            .collect();

        let setup_role = sdp.attributes.get::<Setup>().map(|a| a.0.clone());

        let extensions = sdp.attributes.get_vec::<ExtensionMap>().into_iter().cloned().collect();

        let media_descriptions = sdp
            .media_descriptions
            .iter()
            .map(RtpMediaDescription::from_sdp)
            .collect::<Result<Vec<_>, _>>()?;

        let session = Session {
            id: sdp.origin.session_id,
            version: sdp.origin.session_version,
            bandwidths: sdp.bandwidths.clone(),
            groups,
            ice_lite,
            ice_ufrag,
            ice_pwd,
            ice_options,
            fingerprints,
            setup_role,
            extensions,
            media_descriptions,
        };

        Ok(session)
    }

    pub fn to_sdp(&self) -> crate::sdp::Session {
        todo!()
    }
}

impl FromStr for Session {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let session = crate::sdp::Session::from_str(s)?;
        let session = Self::from_sdp(&session)?;
        Ok(session)
    }
}

impl std::fmt::Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.to_sdp().fmt(f)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum MediaDirection {
    SendOnly,
    ReceiveOnly,
    SendReceive,
    Inactive,
}

// TODO: Include RTX/FEC as part of this rather than their own payloads?
// TODO: If we ignore support for wildcard rtcp-fb attributes, we can include those here.
#[derive(Debug)]
pub struct RtpPayload {
    pub name: String,
    pub clock: u32,
    pub channels: Option<u8>,
    // TODO: Parse H264 profile info
    pub parameters: Option<String>,
}

#[derive(Debug)]
pub struct RtpMediaDescription {
    pub kind: MediaType,
    pub port: u16,
    pub protocol: TransportProtocol,

    pub bandwidths: HashMap<BandwidthType, u64>,

    pub ice_ufrag: Option<String>,
    pub ice_pwd: Option<String>,
    pub ice_options: HashSet<IceOption>,

    pub has_end_of_candidates: bool,
    pub candidates: Vec<Candidate>,

    pub fingerprints: ListOrderedMultimap<FingerprintHashFunction, Vec<u8>>,
    pub setup_role: Option<SetupRole>,

    pub mid: Option<String>,

    pub payloads: HashMap<u8, RtpPayload>,

    pub packet_time: Option<u32>,
    pub max_packet_time: Option<u32>,

    pub direction: MediaDirection,

    // ssrc attributes

    // TODO: Parse these at some level.
    // pub ssrc_attributes: HashMap<u32, HashMap<String, Option<String>>>,

    // ssrc-group seems to have been removed from JSEP?

    // First SSRC of FID / FEC-FR is primary stream, 2nd is the RTX / FEC stream
    // TODO: For those groups we want fast lookup by SSRC
    //       Actually maybe there can only be one group per m-line?
    // pub ssrc_groups: Vec<SsrcGroup>,
    pub extensions: Vec<ExtensionMap>,
    // rtcp-fb, rtcp-mux?, rtcp-mux-only?, rtcp-rsize?
    // msid, imageattr, rid, simulcast
}

impl RtpMediaDescription {
    /// https://rtcweb-wg.github.io/jsep/#rfc.section.5.8
    pub fn from_sdp(sdp: &crate::sdp::MediaDescription) -> Result<Self, String> {
        let ice_ufrag = sdp.attributes.get::<IceUfrag>().map(|a| a.0.clone());
        let ice_pwd = sdp.attributes.get::<IcePwd>().map(|a| a.0.clone());
        let ice_options = sdp
            .attributes
            .get::<IceOptions>()
            .map_or_else(HashSet::new, |a| a.0.clone());

        let has_end_of_candidates = sdp.attributes.get::<EndOfCandidates>().is_some();
        let candidates = sdp.attributes.get_vec::<Candidate>().into_iter().cloned().collect();

        let fingerprints = sdp
            .attributes
            .get_vec::<Fingerprint>()
            .into_iter()
            .map(|a| (a.hash_function.clone(), a.fingerprint.clone()))
            .collect();

        let setup_role = sdp.attributes.get::<Setup>().map(|a| a.0.clone());

        let mid = sdp.attributes.get::<Mid>().map(|a| a.0.clone());

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
            .map(|a| (a.payload, a))
            .collect();

        let payloads: HashMap<_, _> = sdp
            .formats
            .iter()
            .filter_map(|fmt| u8::from_str(fmt).ok())
            .filter_map(|fmt| {
                let map = rtp_maps.get(&fmt)?;
                let parameters = format_parameters.get(&fmt);

                let payload = RtpPayload {
                    name: map.name.clone(),
                    clock: map.clock,
                    channels: map.channels,
                    parameters: parameters.map(|p| p.parameters.clone()),
                };

                Some((fmt, payload))
            })
            .collect();

        let packet_time = sdp.attributes.get::<PacketTime>().map(|a| a.0);
        let max_packet_time = sdp.attributes.get::<MaxPacketTime>().map(|a| a.0);

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

        let extensions = sdp.attributes.get_vec::<ExtensionMap>().into_iter().cloned().collect();

        let media_description = RtpMediaDescription {
            kind: sdp.kind.clone(),
            port: sdp.port,
            protocol: sdp.protocol.clone(),
            bandwidths: sdp.bandwidths.clone(),
            ice_ufrag,
            ice_pwd,
            ice_options,
            has_end_of_candidates,
            candidates,
            fingerprints,
            setup_role,
            mid,
            payloads,
            packet_time,
            max_packet_time,
            direction,
            extensions,
        };

        Ok(media_description)
    }

    pub fn to_sdp(&self) -> crate::sdp::Session {
        todo!()
    }
}
