#![cfg(test)]

use super::*;

const SDP_OFFER: &str = include_str!("../../resources/sdp-offer.txt");
const SDP_ANSWER: &str = include_str!("../../resources/sdp-answer.txt");

#[test]
fn create_offer() {
    let session = Session::new();
    println!("{:#?}", session);
    println!("{}", session);
}

#[test]
fn parse_offer() {
    let session = match Session::from_str(SDP_OFFER) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);
}

#[test]
fn answer_offer() {
    let offer = Session::from_str(SDP_OFFER).unwrap();
    let answer = offer.answer();
    println!("{}", answer);
}

#[test]
fn parse_answer() {
    let session = match Session::from_str(SDP_ANSWER) {
        Ok(session) => session,
        Err(error) => panic!("{}", error),
    };

    println!("{:#?}", session);
}
