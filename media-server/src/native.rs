use media_server_sys as bridge;

mod cxx {
    pub use media_server_sys::UniquePtr;
}

use crate::Result;

use parking_lot::{const_mutex, Mutex};

static INIT_MUTEX: Mutex<bool> = const_mutex(false);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum LoggingLevel {
    None,
    Default,
    Debug,
    UltraDebug,
}

pub fn library_init(logging: LoggingLevel) -> Result<()> {
    let mut is_init = INIT_MUTEX.lock();

    if *is_init {
        return Ok(());
    }

    *is_init = true;

    // TODO: Expose the logging config to consumers.
    bridge::logger_enable_log(logging >= LoggingLevel::Default);
    bridge::logger_enable_debug(logging >= LoggingLevel::Debug);
    bridge::logger_enable_ultra_debug(logging >= LoggingLevel::UltraDebug);

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

pub fn set_port_range(range: Option<(u16, u16)>) -> Result<()> {
    // TODO: It looks like resetting the range to unrestricted may be broken in the library.
    let (min, max) = range.unwrap_or((0, 0));
    bridge::rtp_transport_set_port_range(min, max)?;
    Ok(())
}

pub struct Properties(cxx::UniquePtr<bridge::PropertiesFacade>);

impl Properties {
    pub fn new() -> Self {
        Self(bridge::new_properties())
    }

    pub fn set_int(&mut self, key: &str, value: i32) {
        self.0.pin_mut().set_int(key, value);
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.0.pin_mut().set_bool(key, value);
    }

    pub fn set_string(&mut self, key: &str, value: &str) {
        self.0.pin_mut().set_string(key, value);
    }
}

impl Default for Properties {
    fn default() -> Self {
        Self::new()
    }
}

pub use bridge::DtlsIceTransportListener;

pub type DtlsIceTransportDtlsState = bridge::DtlsIceTransportDtlsState;

pub type MediaFrameType = bridge::MediaFrameType;

pub struct RtpIncomingSourceGroup(cxx::UniquePtr<bridge::RtpIncomingSourceGroupFacade>);

pub struct RtpOutgoingSourceGroup(cxx::UniquePtr<bridge::RtpOutgoingSourceGroupFacade>);

impl RtpOutgoingSourceGroup {
    pub fn add_transponder(&mut self) -> RtpStreamTransponder {
        RtpStreamTransponder(self.0.pin_mut().add_transponder())
    }
}

pub struct RtpStreamTransponder(cxx::UniquePtr<bridge::RtpStreamTransponderFacade>);

impl RtpStreamTransponder {
    pub fn set_incoming(&mut self, incoming: &mut RtpIncomingSourceGroup) {
        self.0.pin_mut().set_incoming(incoming.0.pin_mut());
    }
}

pub struct RtpBundleTransportConnection(cxx::UniquePtr<bridge::RtpBundleTransportConnectionFacade>);

impl RtpBundleTransportConnection {
    pub fn set_listener(&mut self, listener: impl DtlsIceTransportListener + 'static) {
        let listener = bridge::DtlsIceTransportListenerRustAdapter::from(listener);
        self.0.pin_mut().set_listener(Box::new(listener));
    }

    pub fn set_remote_properties(&mut self, properties: &Properties) {
        self.0.pin_mut().set_remote_properties(&properties.0);
    }

    pub fn set_local_properties(&mut self, properties: &Properties) {
        self.0.pin_mut().set_local_properties(&properties.0);
    }

    pub fn add_incoming_source_group(
        &mut self,
        kind: MediaFrameType,
        mid: Option<&str>,
        rid: Option<&str>,
        media_ssrc: Option<u32>,
        rtx_ssrc: Option<u32>,
    ) -> Result<RtpIncomingSourceGroup> {
        let incoming_source_group = self.0.pin_mut().add_incoming_source_group(
            kind,
            mid.unwrap_or(""),
            rid.unwrap_or(""),
            media_ssrc.unwrap_or(0),
            rtx_ssrc.unwrap_or(0),
        )?;

        Ok(RtpIncomingSourceGroup(incoming_source_group))
    }

    pub fn add_outgoing_source_group(
        &mut self,
        kind: MediaFrameType,
        mid: Option<&str>,
        media_ssrc: u32,
        rtx_ssrc: Option<u32>,
    ) -> Result<RtpOutgoingSourceGroup> {
        let outgoing_source_group =
            self.0
                .pin_mut()
                .add_outgoing_source_group(kind, mid.unwrap_or(""), media_ssrc, rtx_ssrc.unwrap_or(0))?;

        Ok(RtpOutgoingSourceGroup(outgoing_source_group))
    }

    pub fn add_remote_candidate(&mut self, ip: &str, port: u16) {
        self.0.pin_mut().add_remote_candidate(ip, port);
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

    pub fn add_ice_transport(
        &mut self,
        username: &str,
        properties: &Properties,
    ) -> Result<RtpBundleTransportConnection> {
        let connection = self.0.pin_mut().add_ice_transport(username, &properties.0)?;
        Ok(RtpBundleTransportConnection(connection))
    }
}
