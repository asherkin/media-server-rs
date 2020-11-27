#[cxx::bridge]
pub mod ffi {
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
        type DtlsIceTransportListener;
        fn on_ice_timeout(self: &mut DtlsIceTransportListener);
        fn on_dtls_state_changed(self: &mut DtlsIceTransportListener, state: DtlsIceTransportDtlsState);
        fn on_remote_ice_candidate_activated(self: &mut DtlsIceTransportListener, ip: &str, port: u16, priority: u32);
    }

    unsafe extern "C++" {
        include!("media-server-rs/include/bridge.h");

        type DtlsConnectionHash;
        type DtlsIceTransportDtlsState;

        fn logger_enable_log(flag: bool);
        fn logger_enable_debug(flag: bool);
        fn logger_enable_ultra_debug(flag: bool);

        fn openssl_class_init();

        fn dtls_connection_initialize() -> i32;
        fn dtls_connection_get_certificate_fingerprint(hash: DtlsConnectionHash) -> String;

        type PropertiesFacade;
        fn new_properties() -> UniquePtr<PropertiesFacade>;
        fn set_int(self: &PropertiesFacade, key: &str, value: i32);
        fn set_bool(self: &PropertiesFacade, key: &str, value: bool);
        fn set_string(self: &PropertiesFacade, key: &str, value: &str);

        type RTPBundleTransportConnectionFacade;
        fn set_listener(self: &RTPBundleTransportConnectionFacade, listener: Box<DtlsIceTransportListener>);

        type RtpBundleTransportFacade;
        fn new_rtp_bundle_transport() -> UniquePtr<RtpBundleTransportFacade>;
        fn init(self: &RtpBundleTransportFacade) -> u16;
        fn add_ice_transport(self: &RtpBundleTransportFacade, username: &str, properties: &PropertiesFacade) -> UniquePtr<RTPBundleTransportConnectionFacade>;
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

// TODO: Implement a way for the callbacks to be user-provided.
pub struct DtlsIceTransportListener();

impl DtlsIceTransportListener {
    fn on_ice_timeout(&mut self) {
        println!("on_ice_timeout()");

        println!("on_ice_timeout() thread: {:?}", std::thread::current());
    }

    fn on_dtls_state_changed(&mut self, state: DtlsIceTransportDtlsState) {
        println!("on_dtls_state_changed(state: {:?})", state);
    }

    fn on_remote_ice_candidate_activated(&mut self, ip: &str, port: u16, priority: u32) {
        println!("on_remote_ice_candidate_activated(ip: {:?}, port: {:?}, priority: {:?})", ip, port, priority);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        println!("main() thread: {:?}", std::thread::current());

        logger_enable_log(true);
        logger_enable_debug(true);
        logger_enable_ultra_debug(true);

        openssl_class_init();

        dtls_connection_initialize();

        let fingerprint = dtls_connection_get_certificate_fingerprint(DtlsConnectionHash::SHA256);
        println!("Fingerprint: {:?}", fingerprint);

        let transport = new_rtp_bundle_transport();

        let port = transport.init();
        println!("Port: {:?}", port);

        // TODO: Create candidates.

        let properties = new_properties();

        properties.set_string("ice.localUsername", "a");
        properties.set_string("ice.localPassword", "b");
        properties.set_string("ice.remoteUsername", "c");
        properties.set_string("ice.remotePassword", "d");

        properties.set_string("dtls.setup", "actpass");
        properties.set_string("dtls.hash", "SHA-256");
        properties.set_string("dtls.fingerprint", &fingerprint); // TODO: Actually the remote one

        properties.set_bool("disableSTUNKeepAlive", false);
        properties.set_string("srtpProtectionProfiles", "");

        let connection = transport.add_ice_transport("test", &properties);
        assert!(!connection.is_null());

        let listener = Box::new(DtlsIceTransportListener());
        connection.set_listener(listener);

        // on_ice_timeout() should fire after 30 seconds.
        std::thread::sleep(std::time::Duration::from_secs(45));
    }
}
