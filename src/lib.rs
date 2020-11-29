mod bridge;

use parking_lot::{Mutex, const_mutex};

// TODO: Figure out an error handling strategy once we have more errors.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static INIT_MUTEX: Mutex<bool> = const_mutex(false);

pub fn library_init() -> Result<()> {
    let mut is_init = INIT_MUTEX.lock();

    if *is_init {
        return Ok(());
    }

    *is_init = true;

    // TODO: Expose the logging config to consumers.
    bridge::logger_enable_log(true);
    bridge::logger_enable_debug(true);
    bridge::logger_enable_ultra_debug(false);

    bridge::openssl_class_init()?;

    // It is unfortunate that this is global state.
    bridge::dtls_connection_initialize()?;

    Ok(())
}

pub enum DtlsConnectionHash {
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    UnknownHash,
}

impl Into<bridge::DtlsConnectionHash> for DtlsConnectionHash {
    fn into(self) -> bridge::DtlsConnectionHash {
        match self {
            DtlsConnectionHash::Sha1 => bridge::DtlsConnectionHash::SHA1,
            DtlsConnectionHash::Sha224 => bridge::DtlsConnectionHash::SHA224,
            DtlsConnectionHash::Sha256 => bridge::DtlsConnectionHash::SHA256,
            DtlsConnectionHash::Sha384 => bridge::DtlsConnectionHash::SHA384,
            DtlsConnectionHash::Sha512 => bridge::DtlsConnectionHash::SHA512,
            DtlsConnectionHash::UnknownHash => bridge::DtlsConnectionHash::UNKNOWN_HASH,
        }
    }
}

pub fn get_certificate_fingerprint(hash: DtlsConnectionHash) -> Result<String> {
    let fingerprint = bridge::dtls_connection_get_certificate_fingerprint(hash.into())?;
    Ok(fingerprint)
}

pub struct Properties(cxx::UniquePtr<bridge::PropertiesFacade>);

impl Properties {
    pub fn new() -> Self {
        Self(bridge::new_properties())
    }

    pub fn set_int(&self, key: &str, value: i32) {
        self.0.set_int(key, value);
    }

    pub fn set_bool(&self, key: &str, value: bool) {
        self.0.set_bool(key, value);
    }

    pub fn set_string(&self, key: &str, value: &str) {
        self.0.set_string(key, value);
    }
}

pub use bridge::DtlsIceTransportListener;

pub type DtlsIceTransportDtlsState = bridge::DtlsIceTransportDtlsState;

pub struct RtpBundleTransportConnection(cxx::UniquePtr<bridge::RtpBundleTransportConnectionFacade>);

impl RtpBundleTransportConnection {
    pub fn set_listener(&self, listener: impl DtlsIceTransportListener + 'static) {
        let listener = bridge::DtlsIceTransportListenerRustAdapter::from(listener);
        self.0.set_listener(Box::new(listener));
    }

    pub fn add_remote_candidate(&self, ip: &str, port: u16) {
        self.0.add_remote_candidate(ip, port);
    }
}

pub struct RtpBundleTransport(cxx::UniquePtr<bridge::RtpBundleTransportFacade>);

impl RtpBundleTransport {
    pub fn new(port: Option<u16>) -> Result<Self> {
        let port = port.unwrap_or(0);
        let transport = bridge::new_rtp_bundle_transport(port)?;
        Ok(Self(transport))
    }

    pub fn get_local_port(&self) -> u16 {
        self.0.get_local_port()
    }

    pub fn add_ice_transport(&self, username: &str, properties: &Properties) -> Result<RtpBundleTransportConnection> {
        let connection = self.0.add_ice_transport(username, &properties.0)?;
        Ok(RtpBundleTransportConnection(connection))
    }
}