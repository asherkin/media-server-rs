use std::time::Duration;

use futures::channel::oneshot;
use futures::future::Either;
use futures_timer::Delay;

use media_server::{
    DtlsConnectionHash, DtlsIceTransportDtlsState, DtlsIceTransportListener, Properties, Result, RtpBundleTransport,
    RtpBundleTransportConnection,
};

struct WaitForConnectionListener(Option<oneshot::Sender<()>>);

impl DtlsIceTransportListener for WaitForConnectionListener {
    fn on_dtls_state_changed(&mut self, state: DtlsIceTransportDtlsState) {
        if state == DtlsIceTransportDtlsState::Connected {
            // Ignore failure, panicking in here is an abort.
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }
}

struct TestTransport {
    transport: RtpBundleTransport,
    connection: RtpBundleTransportConnection,
    receiver: oneshot::Receiver<()>,
}

fn create_test_transport(
    local_username: &str,
    remote_username: &str,
    remote_dtls_setup: &str,
) -> Result<TestTransport> {
    let fingerprint = media_server::get_certificate_fingerprint(DtlsConnectionHash::Sha256)?;

    let transport = RtpBundleTransport::new(None)?;

    let properties = Properties::new();
    properties.set_string("ice.localUsername", local_username);
    properties.set_string("ice.localPassword", "");
    properties.set_string("ice.remoteUsername", remote_username);
    properties.set_string("ice.remotePassword", "");
    properties.set_string("dtls.setup", remote_dtls_setup);
    properties.set_string("dtls.hash", "SHA-256");
    properties.set_string("dtls.fingerprint", &fingerprint);
    properties.set_bool("disableSTUNKeepAlive", true);
    properties.set_string("srtpProtectionProfiles", "");

    let username = local_username.to_owned() + ":" + remote_username;
    let connection = transport.add_ice_transport(username.as_str(), &properties)?;

    let (sender, receiver) = oneshot::channel();
    connection.set_listener(WaitForConnectionListener(Some(sender)));

    Ok(TestTransport {
        transport,
        connection,
        receiver,
    })
}

fn main() -> Result<()> {
    media_server::library_init()?;

    let one = create_test_transport("one", "two", "active")?;
    let two = create_test_transport("two", "one", "passive")?;

    two.connection
        .add_remote_candidate("127.0.0.1", one.transport.get_local_port());

    futures::executor::block_on(async {
        let connected = futures::future::try_join(one.receiver, two.receiver);
        let timeout = Delay::new(Duration::from_secs(5));

        match futures::future::select(connected, timeout).await {
            Either::Left((Ok(_), _)) => Ok(()),
            Either::Left((Err(_), _)) => Err("failed to read from channel"),
            Either::Right(_) => Err("connection timed out"),
        }
    })?;

    println!("WebRTC transports are connected!");

    Ok(())
}
