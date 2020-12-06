use semantic_sdp_derive::SdpEnum;

#[non_exhaustive]
#[derive(Debug, SdpEnum)]
pub enum NetworkType {
    // RFC 4566
    #[sdp("IN")]
    Internet,
}

#[non_exhaustive]
#[derive(Debug, SdpEnum)]
pub enum AddressType {
    // RFC 4566
    #[sdp("IP4")]
    Ip4,
    #[sdp("IP6")]
    Ip6,
}

#[non_exhaustive]
#[derive(Debug, SdpEnum)]
pub enum BandwidthType {
    // RFC 4566
    #[sdp("CT")]
    ConferenceTotal,
    #[sdp("AS")]
    ApplicationSpecific,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, SdpEnum)]
pub enum MediaType {
    // RFC 4566
    #[sdp("audio")]
    Audio,
    #[sdp("video")]
    Video,
    #[sdp("text")]
    Text,
    #[sdp("application")]
    Application,
    #[sdp("message")]
    Message,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, SdpEnum)]
pub enum TransportProtocol {
    // RFC 4566
    #[sdp("udp")]
    Udp,
    #[sdp("RTP/AVP")]
    RtpAvp,
    #[sdp("RTP/SAVP")]
    RtpSavp,

    #[sdp("UDP/TLS/RTP/SAVPF")]
    UdpTlsRtpSavpf,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, SdpEnum)]
pub enum SetupRole {
    // RFC 5763
    #[sdp("active")]
    Active,
    #[sdp("passive")]
    Passive,
    #[sdp("actpass")]
    ActivePassive,
    #[sdp("holdconn")]
    HoldConnection,
}

#[non_exhaustive]
#[derive(Debug, SdpEnum)]
pub enum GroupSemantics {
    // RFC 5888
    #[sdp("LS")]
    LipSynchronization,
    #[sdp("FID")]
    FlowIdentification,

    // draft-ietf-mmusic-sdp-bundle-negotiation
    #[sdp("BUNDLE")]
    Bundle,

    #[sdp(default)]
    Unknown(String),
}
