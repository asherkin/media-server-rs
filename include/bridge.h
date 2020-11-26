#include "RTPBundleTransport.h"

#include "rust/cxx.h"

using DtlsConnectionHash = DTLSConnection::Hash;

void logger_enable_log(bool flag);
void logger_enable_debug(bool flag);
void logger_enable_ultra_debug(bool flag);
void openssl_class_init();
int dtls_connection_initialize();
rust::String dtls_connection_get_certificate_fingerprint(DtlsConnectionHash hash);
std::unique_ptr<RTPBundleTransport> new_rtp_bundle_transport();
