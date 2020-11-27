#pragma once

#include "RTPBundleTransport.h"

#include "rust/cxx.h"

using DTLSConnectionHash = DTLSConnection::Hash;
using DTLSICETransportDTLSState = DTLSICETransport::DTLSState;
using RTPBundleTransportConnection = RTPBundleTransport::Connection;

// TODO: There is *nothing* safe about this.
struct RTPBundleTransportConnectionWrapper {
    RTPBundleTransportConnectionWrapper(RTPBundleTransportConnection *connection):
        connection(connection) {}

    RTPBundleTransportConnection *connection;
};

struct DtlsIceTransportListener;

void logger_enable_log(bool flag);
void logger_enable_debug(bool flag);
void logger_enable_ultra_debug(bool flag);
void openssl_class_init();
int dtls_connection_initialize();
rust::String dtls_connection_get_certificate_fingerprint(DTLSConnectionHash hash);
std::unique_ptr<Properties> new_properties();
void properties_set_int(const std::unique_ptr<Properties> &properties, rust::Str key, int value);
void properties_set_bool(const std::unique_ptr<Properties> &properties, rust::Str key, bool value);
void properties_set_string(const std::unique_ptr<Properties> &properties, rust::Str key, rust::Str value);
std::unique_ptr<RTPBundleTransport> new_rtp_bundle_transport();
std::unique_ptr<RTPBundleTransportConnectionWrapper> rtp_bundle_transport_add_ice_transport(const std::unique_ptr<RTPBundleTransport> &transport, rust::Str username, const Properties &properties);
void rtp_bundle_transport_remove_ice_transport(const std::unique_ptr<RTPBundleTransport> &transport, rust::Str username);
void rtp_bundle_transport_connection_set_listener(const std::unique_ptr<RTPBundleTransportConnectionWrapper> &connection, DtlsIceTransportListener &listener);
