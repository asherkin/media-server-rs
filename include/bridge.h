#pragma once
#include "rust/cxx.h"

#include "DTLSICETransport.h"
#include "RTPBundleTransport.h"

using DtlsConnectionHash = DTLSConnection::Hash;
using DtlsIceTransportDtlsState = DTLSICETransport::DTLSState;

class Properties;

void logger_enable_log(bool flag);
void logger_enable_debug(bool flag);
void logger_enable_ultra_debug(bool flag);

void openssl_class_init();

int dtls_connection_initialize();

rust::String dtls_connection_get_certificate_fingerprint(DtlsConnectionHash hash);

struct PropertiesFacade {
    PropertiesFacade();
    operator const Properties &() const;
    void set_int(rust::Str key, int value) const;
    void set_bool(rust::Str key, bool value) const;
    void set_string(rust::Str key, rust::Str value) const;

private:
    // TODO: We need an indirection layer here due to constness, is there something better?
    std::unique_ptr<Properties> properties;
};

std::unique_ptr<PropertiesFacade> new_properties();

struct DtlsIceTransportListener;
struct DtlsIceTransportListenerAdapter;

struct RTPBundleTransportConnectionFacade {
    RTPBundleTransportConnectionFacade(std::shared_ptr<RTPBundleTransport> transport, std::string username, RTPBundleTransport::Connection *connection);
    ~RTPBundleTransportConnectionFacade();
    void set_listener(rust::Box<DtlsIceTransportListener> listener) const;
    void add_remote_candidate(rust::Str ip, uint16_t port) const;

private:
    std::shared_ptr<RTPBundleTransport> transport;
    std::string username;
    RTPBundleTransport::Connection *connection;
    mutable std::unique_ptr<DtlsIceTransportListenerAdapter> active_listener;
};

struct RtpBundleTransportFacade {
    RtpBundleTransportFacade(uint16_t port = 0);
    uint16_t get_local_port() const;
    std::unique_ptr<RTPBundleTransportConnectionFacade> add_ice_transport(rust::Str username, const PropertiesFacade &properties) const;

private:
    std::shared_ptr<RTPBundleTransport> transport;
};

std::unique_ptr<RtpBundleTransportFacade> new_rtp_bundle_transport(uint16_t port = 0);
