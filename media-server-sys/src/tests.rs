#![cfg(test)]

use super::*;
use futures::channel::oneshot;
use futures::future::Either;
use parking_lot::{const_mutex, Mutex};

static INIT_MUTEX: Mutex<bool> = const_mutex(false);

fn library_init() -> Result<(), Box<dyn std::error::Error>> {
    let mut is_init = INIT_MUTEX.lock();

    if *is_init {
        return Ok(());
    }

    *is_init = true;

    logger_enable_log(true);
    logger_enable_debug(true);
    logger_enable_ultra_debug(false);

    openssl_class_init()?;

    dtls_connection_initialize()?;

    Ok(())
}

#[test]
fn init() {
    library_init().unwrap();

    let fingerprint = dtls_connection_get_certificate_fingerprint(DtlsConnectionHash::SHA256).unwrap();
    println!("Fingerprint: {:?}", fingerprint);
}

#[test]
fn create_transport() {
    library_init().unwrap();

    let transport = new_rtp_bundle_transport(0).unwrap();

    let port = transport.get_local_port();
    println!("Port: {:?}", port);
}

#[test]
fn create_connection_failure() {
    library_init().unwrap();

    let transport = new_rtp_bundle_transport(0).unwrap();

    let properties = new_properties();

    let connection_result = transport.add_ice_transport("invalid", &properties);
    assert!(connection_result.is_err());
}

#[test]
fn transport_connection() {
    library_init().unwrap();

    let fingerprint = dtls_connection_get_certificate_fingerprint(DtlsConnectionHash::SHA256).unwrap();

    let transport_one = new_rtp_bundle_transport(0).unwrap();

    let properties_one = new_properties();
    properties_one.set_string("ice.localUsername", "one");
    properties_one.set_string("ice.localPassword", "one");
    properties_one.set_string("ice.remoteUsername", "two");
    properties_one.set_string("ice.remotePassword", "two");
    properties_one.set_string("dtls.setup", "passive");
    properties_one.set_string("dtls.hash", "SHA-256");
    properties_one.set_string("dtls.fingerprint", &fingerprint);
    properties_one.set_bool("disableSTUNKeepAlive", true);
    properties_one.set_string("srtpProtectionProfiles", "");

    let connection_one = transport_one.add_ice_transport("one:two", &properties_one).unwrap();

    struct LoggingDtlsIceTransportListener(&'static str, Option<oneshot::Sender<()>>);

    impl DtlsIceTransportListener for LoggingDtlsIceTransportListener {
        fn on_ice_timeout(&mut self) {
            println!("{}.on_ice_timeout()", self.0);
        }

        fn on_dtls_state_changed(&mut self, state: DtlsIceTransportDtlsState) {
            println!("{}.on_dtls_state_changed(state: {:?})", self.0, state);

            if state == DtlsIceTransportDtlsState::Connected {
                self.1.take().unwrap().send(()).unwrap();
            }
        }

        fn on_remote_ice_candidate_activated(&mut self, ip: &str, port: u16, priority: u32) {
            println!(
                "{}.on_remote_ice_candidate_activated(ip: {:?}, port: {:?}, priority: {:?})",
                self.0, ip, port, priority
            );
        }
    }

    let (sender_one, receiver_one) = oneshot::channel();
    let listener_one = Box::new(DtlsIceTransportListenerRustAdapter::from(
        LoggingDtlsIceTransportListener("one", Some(sender_one)),
    ));
    connection_one.set_listener(listener_one);

    let transport_two = new_rtp_bundle_transport(0).unwrap();

    let properties_two = new_properties();
    properties_two.set_string("ice.localUsername", "two");
    properties_two.set_string("ice.localPassword", "two");
    properties_two.set_string("ice.remoteUsername", "one");
    properties_two.set_string("ice.remotePassword", "one");
    properties_two.set_string("dtls.setup", "active");
    properties_two.set_string("dtls.hash", "SHA-256");
    properties_two.set_string("dtls.fingerprint", &fingerprint);
    properties_two.set_bool("disableSTUNKeepAlive", true);
    properties_two.set_string("srtpProtectionProfiles", "");

    let connection_two = transport_two.add_ice_transport("two:one", &properties_two).unwrap();

    let (sender_two, receiver_two) = oneshot::channel();
    let listener_two = Box::new(DtlsIceTransportListenerRustAdapter::from(
        LoggingDtlsIceTransportListener("two", Some(sender_two)),
    ));
    connection_two.set_listener(listener_two);

    connection_two.add_remote_candidate("127.0.0.1", transport_one.get_local_port());

    futures::executor::block_on(async {
        let connected = futures::future::try_join(receiver_one, receiver_two);
        let timeout = futures_timer::Delay::new(std::time::Duration::from_secs(10));

        match futures::future::select(connected, timeout).await {
            Either::Left((Ok(_), _)) => println!("connection established"),
            Either::Left((Err(err), _)) => panic!("failed to read from channel: {:?}", err),
            Either::Right(_) => panic!("connection was not established in time"),
        }
    });
}
