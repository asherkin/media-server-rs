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

    unsafe extern "C++" {
        include!("media-server-rs/include/bridge.h");

        fn logger_enable_log(flag: bool);
        fn logger_enable_debug(flag: bool);
        fn logger_enable_ultra_debug(flag: bool);

        fn openssl_class_init();

        type DtlsConnectionHash;

        fn dtls_connection_initialize() -> i32;
        fn dtls_connection_get_certificate_fingerprint(hash: DtlsConnectionHash) -> String;

        type RTPBundleTransport;
        fn new_rtp_bundle_transport() -> UniquePtr<RTPBundleTransport>;

        #[cxx_name="Init"]
        fn init(self: Pin<&mut RTPBundleTransport>) -> i32;
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi::*;

    #[test]
    fn init() {
        logger_enable_log(true);
        logger_enable_debug(true);
        logger_enable_ultra_debug(true);

        openssl_class_init();

        dtls_connection_initialize();

        println!("Fingerprint: {:?}", dtls_connection_get_certificate_fingerprint(DtlsConnectionHash::SHA256));

        let mut transport = new_rtp_bundle_transport();
        let transport = transport.pin_mut();
        // println!("Transport: {:?}", transport);

        let port = transport.init();
        println!("Port: {:?}", port);
    }
}
