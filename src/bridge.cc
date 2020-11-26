#include "media-server-rs/include/bridge.h"

#include "log.h"
#include "OpenSSL.h"

void logger_enable_log(bool flag) {
    Logger::EnableLog(flag);
}

void logger_enable_debug(bool flag) {
    Logger::EnableDebug(flag);
}

void logger_enable_ultra_debug(bool flag) {
    Logger::EnableUltraDebug(flag);
}

void openssl_class_init() {
    OpenSSL::ClassInit();
}

int dtls_connection_initialize() {
    return DTLSConnection::Initialize();
}

rust::String dtls_connection_get_certificate_fingerprint(DTLSConnection::Hash hash) {
    return DTLSConnection::GetCertificateFingerPrint(hash);
}

std::unique_ptr<RTPBundleTransport> new_rtp_bundle_transport() {
    return std::make_unique<RTPBundleTransport>();
}
