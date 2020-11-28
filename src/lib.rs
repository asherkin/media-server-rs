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
pub trait DtlsIceTransportListenerTrait: Send {
    fn into_listener(self) -> Box<DtlsIceTransportListener> where Self: Sized + 'static {
        Box::new(DtlsIceTransportListener(Box::new(self)))
    }

    fn on_ice_timeout(&mut self) {}
    fn on_dtls_state_changed(&mut self, state: DtlsIceTransportDtlsState) {}
    fn on_remote_ice_candidate_activated(&mut self, ip: &str, port: u16, priority: u32) {}
}

pub struct DtlsIceTransportListener(Box<dyn DtlsIceTransportListenerTrait>);

impl DtlsIceTransportListener {
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

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::{Mutex, const_mutex};
    use futures::future::Either;

    static INIT_MUTEX: Mutex<bool> = const_mutex(false);

    fn prepare_library() {
        println!("main() thread: {:?}", std::thread::current());

        let mut is_init = INIT_MUTEX.lock();
        if *is_init {
            println!("library already initialised");
            return;
        } else {
            *is_init = true;
        }

        logger_enable_log(true);
        logger_enable_debug(true);
        logger_enable_ultra_debug(true);

        openssl_class_init();

        // It is unfortunate that this is global state.
        dtls_connection_initialize();

        println!("library initialisation done");
    }

    #[test]
    fn init() {
        prepare_library();

        let fingerprint = dtls_connection_get_certificate_fingerprint(DtlsConnectionHash::SHA256);
        println!("Fingerprint: {:?}", fingerprint);
    }

    #[test]
    fn create_transport() {
        prepare_library();

        let transport = new_rtp_bundle_transport(0).unwrap();

        let port = transport.get_local_port();
        println!("Port: {:?}", port);
    }

    #[test]
    fn create_connection_failure() {
        prepare_library();

        let transport = new_rtp_bundle_transport(0).unwrap();

        let properties = new_properties();

        let connection_result = transport.add_ice_transport("invalid", &properties);
        assert!(connection_result.is_err());
    }

    #[test]
    fn create_connection_callbacks() {
        prepare_library();

        let transport = new_rtp_bundle_transport(0).unwrap();

        let properties = new_properties();

        properties.set_string("ice.localUsername", "a");
        properties.set_string("ice.localPassword", "b");
        properties.set_string("ice.remoteUsername", "c");
        properties.set_string("ice.remotePassword", "d");

        properties.set_string("dtls.setup", "actpass");
        properties.set_string("dtls.hash", "SHA-256");
        properties.set_string("dtls.fingerprint", "");

        properties.set_bool("disableSTUNKeepAlive", false);
        properties.set_string("srtpProtectionProfiles", "");

        let connection = transport.add_ice_transport("test", &properties).unwrap();
        assert!(!connection.is_null());

        struct IceTimeoutListener(Option<futures::channel::oneshot::Sender<()>>);

        impl DtlsIceTransportListenerTrait for IceTimeoutListener {
            fn on_ice_timeout(&mut self) {
                println!("on_ice_timeout() thread: {:?}", std::thread::current());
                let sender = self.0.take().unwrap();
                sender.send(()).unwrap();
            }
        }

        let (sender, receiver) = futures::channel::oneshot::channel();

        let listener = IceTimeoutListener(Some(sender)).into_listener();
        connection.set_listener(listener);

        // on_ice_timeout() should fire after 30 seconds.
        futures::executor::block_on(async {
            println!("{:?}: Waiting for on_ice_timeout", std::thread::current());
            let timeout = futures_timer::Delay::new(std::time::Duration::from_secs(45));
            match futures::future::select(receiver, timeout).await {
                Either::Left((Ok(_), _)) => println!("on_ice_timeout triggered"),
                Either::Left((Err(err), _)) => panic!("failed to read from channel: {:?}", err),
                Either::Right(_) => panic!("on_ice_timeout was not triggered in time"),
            }
        });
    }
}
