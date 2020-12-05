#![cfg(test)]

use super::*;

const SDP_OFFER: &str = include_str!("../resources/sdp-offer.txt");
const SDP_ANSWER: &str = include_str!("../resources/sdp-answer.txt");

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
