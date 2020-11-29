#![cfg(test)]

const SDP_OFFER: &str = include_str!("../resources/sdp-offer.txt");
const SDP_ANSWER: &str = include_str!("../resources/sdp-answer.txt");

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);

    println!("{}", SDP_OFFER);
    println!("{}", SDP_ANSWER);
}
