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
fn parse_offer_chrome_rid() {
    let session = match UnifiedBundleSession::from_str(SDP_OFFER_CHROME_RID) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);
}

#[test]
fn answer_offer() {
    let offer = UnifiedBundleSession::from_str(SDP_OFFER).unwrap();
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
