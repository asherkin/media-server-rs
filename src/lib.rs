#[cxx::bridge]
pub mod ffi {
    #[repr(i32)]
    enum DTLSConnectionHash {
        SHA1,
        SHA224,
        SHA256,
        SHA384,
        SHA512,
        UNKNOWN_HASH,
    }

    #[repr(i32)]
    enum DTLSICETransportDTLSState {
        New,
        Connecting,
        Connected,
        Closed,
        Failed,
    }

    extern "Rust" {
        type DtlsIceTransportListener;
        fn on_ice_timeout(self: &mut DtlsIceTransportListener);
        fn on_dtls_state_changed(self: &mut DtlsIceTransportListener, state: DTLSICETransportDTLSState);
        fn on_remote_ice_candidate_activated(self: &mut DtlsIceTransportListener, ip: &str, port: u16, priority: u32);
    }

    unsafe extern "C++" {
        include!("media-server-rs/include/bridge.h");

        fn logger_enable_log(flag: bool);
        fn logger_enable_debug(flag: bool);
        fn logger_enable_ultra_debug(flag: bool);

        fn openssl_class_init();

        type DTLSConnectionHash;
        type DTLSICETransportDTLSState;

        fn dtls_connection_initialize() -> i32;
        fn dtls_connection_get_certificate_fingerprint(hash: DTLSConnectionHash) -> String;

        type Properties;
        fn new_properties() -> UniquePtr<Properties>;
        fn properties_set_int(properties: &UniquePtr<Properties>, key: &str, value: i32);
        fn properties_set_bool(properties: &UniquePtr<Properties>, key: &str, value: bool);
        fn properties_set_string(properties: &UniquePtr<Properties>, key: &str, value: &str);

        type RTPBundleTransportConnectionWrapper;
        fn rtp_bundle_transport_connection_set_listener(connection: &UniquePtr<RTPBundleTransportConnectionWrapper>, listener: &mut DtlsIceTransportListener);

        type RTPBundleTransport;
        // TODO: Should we just do the init in here?
        fn new_rtp_bundle_transport() -> UniquePtr<RTPBundleTransport>;
        fn rtp_bundle_transport_add_ice_transport(transport: &UniquePtr<RTPBundleTransport>, username: &str, properties: &Properties) -> UniquePtr<RTPBundleTransportConnectionWrapper>;
        fn rtp_bundle_transport_remove_ice_transport(transport: &UniquePtr<RTPBundleTransport>, username: &str);

        #[cxx_name="Init"]
        fn init(self: Pin<&mut RTPBundleTransport>) -> i32;
    }
}

pub use ffi::*;

impl std::fmt::Debug for DTLSICETransportDTLSState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &DTLSICETransportDTLSState::New => f.write_str("New"),
            &DTLSICETransportDTLSState::Connecting => f.write_str("Connecting"),
            &DTLSICETransportDTLSState::Connected => f.write_str("Connected"),
            &DTLSICETransportDTLSState::Closed => f.write_str("Closed"),
            &DTLSICETransportDTLSState::Failed => f.write_str("Failed"),
            _ => f.write_str("Unknown"),
        }
    }
}

// TODO: Implement a way for the callbacks to be user-provided.
pub struct DtlsIceTransportListener();

impl DtlsIceTransportListener {
    fn on_ice_timeout(&mut self) {
        println!("on_ice_timeout()");
    }

    fn on_dtls_state_changed(&mut self, state: DTLSICETransportDTLSState) {
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
        logger_enable_log(true);
        logger_enable_debug(true);
        logger_enable_ultra_debug(true);

        openssl_class_init();

        dtls_connection_initialize();

        let fingerprint = dtls_connection_get_certificate_fingerprint(DTLSConnectionHash::SHA256);
        println!("Fingerprint: {:?}", fingerprint);

        let mut transport = new_rtp_bundle_transport();

        let port = transport.pin_mut().init();
        println!("Port: {:?}", port);

        // TODO: Create candidates.

        let properties = new_properties();

        properties_set_string(&properties, "ice.localUsername", "a");
        properties_set_string(&properties, "ice.localPassword", "b");
        properties_set_string(&properties, "ice.remoteUsername", "c");
        properties_set_string(&properties, "ice.remotePassword", "d");

        properties_set_string(&properties, "dtls.setup", "actpass");
        properties_set_string(&properties, "dtls.hash", "SHA-256");
        properties_set_string(&properties, "dtls.fingerprint", &fingerprint); // TODO: Actually the remote one

        properties_set_bool(&properties, "disableSTUNKeepAlive", false);
        properties_set_string(&properties, "srtpProtectionProfiles", "");

        let connection = rtp_bundle_transport_add_ice_transport(&transport, "test", &properties);
        assert!(!connection.is_null());

        let mut listener = DtlsIceTransportListener();
        rtp_bundle_transport_connection_set_listener(&connection, &mut listener);

        // on_ice_timeout() should fire after 30 seconds.
        std::thread::sleep(std::time::Duration::from_secs(45));

        rtp_bundle_transport_remove_ice_transport(&transport, "test");
    }
}
