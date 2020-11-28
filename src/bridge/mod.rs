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

    extern "Rust" {
        type DtlsIceTransportListenerRustAdapter;
        fn on_ice_timeout(self: &mut DtlsIceTransportListenerRustAdapter);
        fn on_dtls_state_changed(self: &mut DtlsIceTransportListenerRustAdapter, state: DtlsIceTransportDtlsState);
        fn on_remote_ice_candidate_activated(self: &mut DtlsIceTransportListenerRustAdapter, ip: &str, port: u16, priority: u32);
    }

    unsafe extern "C++" {
        include!("media-server/include/bridge.h");

        type DtlsConnectionHash;
        type DtlsIceTransportDtlsState;

        fn logger_enable_log(flag: bool);
        fn logger_enable_debug(flag: bool);
        fn logger_enable_ultra_debug(flag: bool);

        fn openssl_class_init() -> Result<()>;

        fn dtls_connection_initialize() -> Result<()>;
        fn dtls_connection_get_certificate_fingerprint(hash: DtlsConnectionHash) -> Result<String>;

        type PropertiesFacade;
        fn new_properties() -> UniquePtr<PropertiesFacade>;
        fn set_int(self: &PropertiesFacade, key: &str, value: i32);
        fn set_bool(self: &PropertiesFacade, key: &str, value: bool);
        fn set_string(self: &PropertiesFacade, key: &str, value: &str);

        type RTPBundleTransportConnectionFacade;
        fn set_listener(self: &RTPBundleTransportConnectionFacade, listener: Box<DtlsIceTransportListenerRustAdapter>);
        fn add_remote_candidate(self: &RTPBundleTransportConnectionFacade, ip: &str, port: u16);

        type RtpBundleTransportFacade;
        fn new_rtp_bundle_transport(port: u16) -> Result<UniquePtr<RtpBundleTransportFacade>>;
        fn get_local_port(self: &RtpBundleTransportFacade) -> u16;
        fn add_ice_transport(self: &RtpBundleTransportFacade, username: &str, properties: &PropertiesFacade) -> Result<UniquePtr<RTPBundleTransportConnectionFacade>>;
    }
}

pub use ffi::*;

impl std::fmt::Debug for DtlsIceTransportDtlsState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &DtlsIceTransportDtlsState::New => f.write_str("New"),
            &DtlsIceTransportDtlsState::Connecting => f.write_str("Connecting"),
            &DtlsIceTransportDtlsState::Connected => f.write_str("Connected"),
            &DtlsIceTransportDtlsState::Closed => f.write_str("Closed"),
            &DtlsIceTransportDtlsState::Failed => f.write_str("Failed"),
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

impl<T> From<T> for DtlsIceTransportListenerRustAdapter where T: 'static + DtlsIceTransportListener {
    fn from(listener: T) -> Self {
        Self(Box::new(listener))
    }
}
