mod tests;

#[cxx::bridge]
mod ffi {
    #[repr(i32)]
    enum DtlsConnectionHash {
        SHA1,
        SHA224,
        SHA256,
        SHA384,
        SHA512,
        UNKNOWN_HASH,
    }

    #[repr(i32)]
    enum DtlsIceTransportDtlsState {
        New,
        Connecting,
        Connected,
        Closed,
        Failed,
    }

    #[repr(i32)]
    #[derive(Debug, Copy, Clone)]
    enum MediaFrameType {
        Unknown = -1,
        Audio,
        Video,
        Text,
    }

    extern "Rust" {
        type DtlsIceTransportListenerRustAdapter;
        fn on_ice_timeout(self: &mut DtlsIceTransportListenerRustAdapter);
        fn on_dtls_state_changed(self: &mut DtlsIceTransportListenerRustAdapter, state: DtlsIceTransportDtlsState);
        fn on_remote_ice_candidate_activated(
            self: &mut DtlsIceTransportListenerRustAdapter,
            ip: &str,
            port: u16,
            priority: u32,
        );
    }

    unsafe extern "C++" {
        include!("media-server-sys/include/bridge.h");

        type DtlsConnectionHash;
        type DtlsIceTransportDtlsState;
        type MediaFrameType;

        fn logger_enable_log(flag: bool);
        fn logger_enable_debug(flag: bool);
        fn logger_enable_ultra_debug(flag: bool);

        fn openssl_class_init() -> Result<()>;

        fn dtls_connection_initialize() -> Result<()>;
        fn dtls_connection_get_certificate_fingerprint(hash: DtlsConnectionHash) -> Result<String>;

        fn rtp_transport_set_port_range(min: u16, max: u16) -> Result<()>;

        type PropertiesFacade;
        fn new_properties() -> UniquePtr<PropertiesFacade>;
        fn set_int(self: Pin<&mut PropertiesFacade>, key: &str, value: i32);
        fn set_bool(self: Pin<&mut PropertiesFacade>, key: &str, value: bool);
        fn set_string(self: Pin<&mut PropertiesFacade>, key: &str, value: &str);

        type RtpIncomingSourceGroupFacade;

        type RtpOutgoingSourceGroupFacade;
        fn add_transponder(self: Pin<&mut RtpOutgoingSourceGroupFacade>) -> UniquePtr<RtpStreamTransponderFacade>;

        type RtpStreamTransponderFacade;
        fn set_incoming(self: Pin<&mut RtpStreamTransponderFacade>, incoming: Pin<&mut RtpIncomingSourceGroupFacade>);

        type RtpBundleTransportConnectionFacade;
        fn set_listener(
            self: Pin<&mut RtpBundleTransportConnectionFacade>,
            listener: Box<DtlsIceTransportListenerRustAdapter>,
        );
        fn set_remote_properties(self: Pin<&mut RtpBundleTransportConnectionFacade>, properties: &PropertiesFacade);
        fn set_local_properties(self: Pin<&mut RtpBundleTransportConnectionFacade>, properties: &PropertiesFacade);
        fn add_incoming_source_group(
            self: Pin<&mut RtpBundleTransportConnectionFacade>,
            kind: MediaFrameType,
            mid: &str,
            rid: &str,
            media_ssrc: u32,
            rtx_ssrc: u32,
        ) -> Result<UniquePtr<RtpIncomingSourceGroupFacade>>;
        fn add_outgoing_source_group(
            self: Pin<&mut RtpBundleTransportConnectionFacade>,
            kind: MediaFrameType,
            mid: &str,
            media_ssrc: u32,
            rtx_ssrc: u32,
        ) -> Result<UniquePtr<RtpOutgoingSourceGroupFacade>>;
        fn add_remote_candidate(self: Pin<&mut RtpBundleTransportConnectionFacade>, ip: &str, port: u16);

        type RtpBundleTransportFacade;
        fn new_rtp_bundle_transport(port: u16) -> Result<UniquePtr<RtpBundleTransportFacade>>;
        fn get_local_port(self: &RtpBundleTransportFacade) -> u16;
        fn add_ice_transport(
            self: Pin<&mut RtpBundleTransportFacade>,
            username: &str,
            properties: &PropertiesFacade,
        ) -> Result<UniquePtr<RtpBundleTransportConnectionFacade>>;
    }
}

pub use cxx::UniquePtr;
pub use ffi::*;

unsafe impl Send for PropertiesFacade {}
unsafe impl Send for RtpIncomingSourceGroupFacade {}
unsafe impl Send for RtpOutgoingSourceGroupFacade {}
unsafe impl Send for RtpStreamTransponderFacade {}
unsafe impl Send for RtpBundleTransportConnectionFacade {}
unsafe impl Send for RtpBundleTransportFacade {}

impl std::fmt::Debug for DtlsIceTransportDtlsState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DtlsIceTransportDtlsState::New => f.write_str("New"),
            DtlsIceTransportDtlsState::Connecting => f.write_str("Connecting"),
            DtlsIceTransportDtlsState::Connected => f.write_str("Connected"),
            DtlsIceTransportDtlsState::Closed => f.write_str("Closed"),
            DtlsIceTransportDtlsState::Failed => f.write_str("Failed"),
            _ => f.write_str("Unknown"),
        }
    }
}

#[allow(unused_variables)]
pub trait DtlsIceTransportListener: Send {
    fn on_ice_timeout(&mut self) {}
    fn on_dtls_state_changed(&mut self, state: DtlsIceTransportDtlsState) {}
    fn on_remote_ice_candidate_activated(&mut self, ip: &str, port: u16, priority: u32) {}
}

pub struct DtlsIceTransportListenerRustAdapter(Box<dyn DtlsIceTransportListener>);

impl DtlsIceTransportListenerRustAdapter {
    fn on_ice_timeout(&mut self) {
        self.0.on_ice_timeout()
    }

    fn on_dtls_state_changed(&mut self, state: DtlsIceTransportDtlsState) {
        self.0.on_dtls_state_changed(state)
    }

    fn on_remote_ice_candidate_activated(&mut self, ip: &str, port: u16, priority: u32) {
        self.0.on_remote_ice_candidate_activated(ip, port, priority)
    }
}

impl<T> From<T> for DtlsIceTransportListenerRustAdapter
where
    T: 'static + DtlsIceTransportListener,
{
    fn from(listener: T) -> Self {
        Self(Box::new(listener))
    }
}
