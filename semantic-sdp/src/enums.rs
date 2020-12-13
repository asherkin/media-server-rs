use semantic_sdp_derive::SdpEnum;

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum NetworkType {
    // RFC 4566
    #[sdp("IN")]
    Internet,
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum AddressType {
    // RFC 4566
    #[sdp("IP4")]
    Ip4,
    #[sdp("IP6")]
    Ip6,
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum BandwidthType {
    // RFC 3556 / 4566
    #[sdp("CT")]
    ConferenceTotal,
    #[sdp("AS")]
    ApplicationSpecific,

    // RFC 3890
    #[sdp("TIAS")]
    TransportSpecific,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
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
#[derive(Debug, Clone, SdpEnum)]
pub enum TransportProtocol {
    // RFC 4566
    #[sdp("udp")]
    Udp,
    #[sdp("RTP/AVP")]
    RtpAvp,

    // RFC 4585
    #[sdp("RTP/AVPF")]
    RtpAvpf,

    // RFC 3711
    #[sdp("RTP/SAVP")]
    RtpSavp,

    // RFC 5124
    #[sdp("RTP/SAVPF")]
    RtpSavpf,

    // RFC 7850
    #[sdp("TCP/TLS/RTP/SAVP")]
    TcpTlsRtpSavp,
    #[sdp("TCP/TLS/RTP/SAVPF")]
    TcpTlsRtpSavpf,

    // RFC 5764
    #[sdp("UDP/TLS/RTP/SAVP")]
    UdpTlsRtpSavp,
    #[sdp("UDP/TLS/RTP/SAVPF")]
    UdpTlsRtpSavpf,

    //
    #[sdp("UDP/DTLS/SCTP")]
    UdpDtlsSctp,
    #[sdp("TCP/DTLS/SCTP")]
    TcpDtlsSctp,
    #[sdp("DTLS/SCTP")]
    DtlsSctp,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum IceTransportType {
    // RFC 5245
    #[sdp("UDP")]
    Udp,

    // RFC 6544
    #[sdp("TCP")]
    Tcp,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum IceCandidateType {
    // RFC 5245
    #[sdp("host")]
    Host,
    #[sdp("srflx")]
    ServerReflexive,
    #[sdp("prflx")]
    PeerReflexive,
    #[sdp("relay")]
    Relayed,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum IceOption {
    // draft-ietf-ice-trickle-21
    #[sdp("trickle")]
    Trickle,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum IceTcpType {
    // RFC 6544
    #[sdp("active")]
    Active,
    #[sdp("passive")]
    Passive,
    #[sdp("so")]
    SimultaneousOpen,
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum SetupRole {
    // RFC 4145 / RFC 5763
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
#[derive(Debug, Clone, SdpEnum)]
pub enum FingerprintHashFunction {
    // RFC 3279 / RFC 4572 / RFC 5763
    #[sdp("sha-1")]
    Sha1,
    #[sdp("sha-224")]
    Sha224,
    #[sdp("sha-256")]
    Sha256,
    #[sdp("sha-384")]
    Sha384,
    #[sdp("sha-512")]
    Sha512,
    #[sdp("md5")]
    Md5,
    #[sdp("md2")]
    Md2,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
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

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum SsrcGroupSemantics {
    // RFC 5888 / RFC 5576
    #[sdp("FID")]
    FlowIdentification,
    #[sdp("FEC")]
    ForwardErrorCorrection,

    #[sdp(default)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum ExtensionMapDirection {
    // RFC 8285
    #[sdp("sendonly")]
    SendOnly,
    #[sdp("recvonly")]
    ReceiveOnly,
    #[sdp("sendrecv")]
    SendReceive,
    #[sdp("inactive")]
    Inactive,
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum RidDirection {
    // draft-ietf-mmusic-rid
    #[sdp("send")]
    Send,
    #[sdp("recv")]
    Receive,
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
pub enum RtpCodecName {
    // Audio
    #[sdp("PCMA")]
    Pcma,
    #[sdp("PCMU")]
    Pcmu,
    #[sdp("G722")]
    G722,
    #[sdp("opus")]
    Opus,

    #[sdp("CN")]
    ComfortNoise,
    #[sdp("telephone-event")]
    TelephoneEvent,

    // Video
    #[sdp("H264")]
    H264,
    #[sdp("VP8")]
    Vp8,
    #[sdp("VP9")]
    Vp9,

    // Repaired / Redundant Data
    #[sdp("rtx")]
    Rtx,
    #[sdp("red")]
    Red,
    #[sdp("ulpfec")]
    UlpFec,
    #[sdp("flexfec")]
    FlexFec,

    #[sdp(default)]
    Unknown(String),
}
