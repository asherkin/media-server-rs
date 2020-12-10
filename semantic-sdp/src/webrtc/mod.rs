//! This module implements the SDP parts of the JSEP specification.
//!
//! https://rtcweb-wg.github.io/jsep/#rfc.section.5

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use ordered_multimap::ListOrderedMultimap;
use rand::Rng;

use crate::attributes::{
    Candidate, EndOfCandidates, ExtensionMap, Fingerprint, Group, IceLite, IceOptions, IcePwd, IceUfrag, Setup,
};
use crate::enums::{BandwidthType, FingerprintHashFunction, IceOption, SetupRole};

mod tests;

// TODO: Consider adding a set of newtypes for the common ones we don't want mixed up,
//       e.g. fingerprints, SSRCs, MIDs, RIDs

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

#[derive(Debug)]
pub struct RtpMediaDescription {
    pub bandwidths: HashMap<BandwidthType, u64>,

    pub ice_ufrag: Option<String>,
    pub ice_pwd: Option<String>,
    pub ice_options: HashSet<IceOption>,

    pub has_end_of_candidates: bool,
    pub candidates: Vec<Candidate>,

    pub fingerprints: ListOrderedMultimap<FingerprintHashFunction, Vec<u8>>,
    pub setup_role: Option<SetupRole>,

    // packet types (fmts) from the m-line
    // rtpmap, fmtp, ptime, maxptime
    // which direction attribute, sendrecv if none
    // ssrc attributes
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

        let extensions = sdp.attributes.get_vec::<ExtensionMap>().into_iter().cloned().collect();

        let media_description = RtpMediaDescription {
            bandwidths: sdp.bandwidths.clone(),
            ice_ufrag,
            ice_pwd,
            ice_options,
            has_end_of_candidates,
            candidates,
            fingerprints,
            setup_role,
            extensions,
        };

        Ok(media_description)
    }

    pub fn to_sdp(&self) -> crate::sdp::Session {
        todo!()
    }
}
