#![cfg(test)]

use super::*;

const SDP_OFFER: &str = include_str!("../../resources/sdp-offer.txt");
const SDP_OFFER_CHROME_SSRC: &str = include_str!("../../resources/sdp-offer-chrome-ssrc.txt");
const SDP_OFFER_CHROME_RID: &str = include_str!("../../resources/sdp-offer-chrome-rid.txt");
const SDP_ANSWER: &str = include_str!("../../resources/sdp-answer.txt");

#[test]
fn create_offer() {
    let session = UnifiedBundleSession::new();
    println!("{:#?}", session);
    println!("{}", session);
}

#[test]
fn parse_offer() {
    let session = match UnifiedBundleSession::from_str(SDP_OFFER) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);
}

#[test]
fn parse_offer_chrome_ssrc() {
    let session = match UnifiedBundleSession::from_str(SDP_OFFER_CHROME_SSRC) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);
}

#[test]
#[ignore]
fn parse_and_serialize_offer_chrome_ssrc() {
    let parsed = UnifiedBundleSession::from_str(SDP_OFFER_CHROME_SSRC).unwrap();
    let serialized = parsed.to_string();
    // This is expected to fail, some things are in a different order.
    assert_eq!(SDP_OFFER_CHROME_SSRC, serialized);
}

#[test]
fn parse_offer_chrome_rid() {
    let session = match UnifiedBundleSession::from_str(SDP_OFFER_CHROME_RID) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);
}

#[test]
#[ignore]
fn parse_and_serialize_offer_chrome_rid() {
    let parsed = UnifiedBundleSession::from_str(SDP_OFFER_CHROME_RID).unwrap();
    let serialized = parsed.to_string();
    // This is expected to fail, some things are in a different order.
    assert_eq!(SDP_OFFER_CHROME_RID, serialized);
}

#[test]
fn answer_offer() {
    let offer = UnifiedBundleSession::from_str(SDP_OFFER).unwrap();
    let answer = offer.answer();
    println!("{}", answer);
}

#[test]
fn answer_offer_chrome_ssrc() {
    let offer = UnifiedBundleSession::from_str(SDP_OFFER_CHROME_SSRC).unwrap();
    let answer = offer.answer();
    println!("{}", answer);
}

#[test]
fn answer_offer_chrome_rid() {
    let offer = UnifiedBundleSession::from_str(SDP_OFFER_CHROME_RID).unwrap();
    let answer = offer.answer();
    println!("{}", answer);
}

#[test]
fn parse_answer() {
    let session = match UnifiedBundleSession::from_str(SDP_ANSWER) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);
}
