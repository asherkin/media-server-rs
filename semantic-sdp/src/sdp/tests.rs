#![cfg(test)]

use std::collections::HashSet;

use crate::attributes::{Group, IceLite};

use super::*;

const SDP_OFFER: &str = include_str!("../../resources/sdp-offer.txt");
const SDP_ANSWER: &str = include_str!("../../resources/sdp-answer.txt");

#[test]
fn parse_offer() {
    let session = match Session::from_str(SDP_OFFER) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);

    assert_eq!(session.origin.session_id, 6842575828159820380);
    assert_eq!(session.name, None);

    let group = session.attributes.get::<Group>().unwrap();
    println!("group: {:#?}", group);

    assert_eq!(group.semantics, GroupSemantics::Bundle);
}

#[test]
fn parse_and_serialize_offer() {
    let parsed = Session::from_str(SDP_OFFER).unwrap();
    let serialized = parsed.to_string();
    assert_eq!(SDP_OFFER, serialized);
}

#[test]
fn parse_answer() {
    let session = match Session::from_str(SDP_ANSWER) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);

    assert_eq!(session.origin.session_id, 1606654361284);
    assert_eq!(session.name, Some("semantic-sdp".to_owned()));

    session.attributes.get::<IceLite>().unwrap();
}

#[test]
fn parse_and_serialize_answer() {
    let parsed = Session::from_str(SDP_ANSWER).unwrap();
    let serialized = parsed.to_string();
    assert_eq!(SDP_ANSWER, serialized);
}

#[test]
fn unknown_attributes() {
    fn get_unknown(attributes: &AttributeMap) -> HashSet<String> {
        let mut unknown = HashSet::new();

        for (name, attribute) in attributes {
            let value: Option<&Option<String>> = attribute.as_any().downcast_ref();
            if value.is_some() {
                unknown.insert(name.to_owned());
            }
        }

        unknown
    }

    fn get_unknown_sdp(sdp: &str) -> HashSet<String> {
        let session = Session::from_str(sdp).unwrap();

        let mut unknown = get_unknown(&session.attributes);

        for media in &session.media_descriptions {
            unknown.extend(get_unknown(&media.attributes));
        }

        unknown
    }

    let mut unknown = HashSet::new();
    unknown.extend(get_unknown_sdp(SDP_OFFER));
    unknown.extend(get_unknown_sdp(SDP_ANSWER));

    println!("{:#?}", unknown);

    assert_eq!(unknown.len(), 0);
}
