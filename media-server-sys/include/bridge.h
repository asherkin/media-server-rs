#pragma once
#include "rust/cxx.h"

#include "DTLSICETransport.h"
#include "RTPBundleTransport.h"

using DtlsConnectionHash = DTLSConnection::Hash;
using DtlsIceTransportDtlsState = DTLSICETransport::DTLSState;
using MediaFrameType = MediaFrame::Type;

class Properties;

void logger_enable_log(bool flag);
void logger_enable_debug(bool flag);
void logger_enable_ultra_debug(bool flag);

void openssl_class_init();

void dtls_connection_initialize();

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

struct DtlsIceTransportListenerRustAdapter;
struct DtlsIceTransportListenerCxxAdapter;

struct OwnedRtpBundleTransportConnection {
    OwnedRtpBundleTransportConnection(std::shared_ptr<RTPBundleTransport> transport, RTPBundleTransport::Connection *connection);
    ~OwnedRtpBundleTransportConnection();
    RTPBundleTransport::Connection *operator->();

private:
    std::shared_ptr<RTPBundleTransport> transport;
    RTPBundleTransport::Connection *connection;
};

struct RtpIncomingSourceGroupFacade {
    RtpIncomingSourceGroupFacade(std::shared_ptr<OwnedRtpBundleTransportConnection> connection, std::unique_ptr<RTPIncomingSourceGroup> source_group);
    ~RtpIncomingSourceGroupFacade();

private:
    std::shared_ptr<OwnedRtpBundleTransportConnection> connection;
    std::unique_ptr<RTPIncomingSourceGroup> source_group;
};

struct RtpBundleTransportConnectionFacade {
    RtpBundleTransportConnectionFacade(std::shared_ptr<RTPBundleTransport> transport, std::shared_ptr<OwnedRtpBundleTransportConnection> connection);
    ~RtpBundleTransportConnectionFacade();
    void set_listener(rust::Box<DtlsIceTransportListenerRustAdapter> listener) const;
    void set_remote_properties(const PropertiesFacade &properties) const;
    void set_local_properties(const PropertiesFacade &properties) const;
    std::unique_ptr<RtpIncomingSourceGroupFacade> add_incoming_source_group(MediaFrameType type, rust::Str mid, rust::Str rid, uint32_t mediaSsrc, uint32_t rtxSsrc) const;
    void add_remote_candidate(rust::Str ip, uint16_t port) const;

private:
    std::shared_ptr<RTPBundleTransport> transport;
    std::shared_ptr<OwnedRtpBundleTransportConnection> connection;
    mutable std::unique_ptr<DtlsIceTransportListenerCxxAdapter> active_listener;
};

struct RtpBundleTransportFacade {
    RtpBundleTransportFacade(uint16_t port = 0);
    uint16_t get_local_port() const;
    std::unique_ptr<RtpBundleTransportConnectionFacade> add_ice_transport(rust::Str username, const PropertiesFacade &properties) const;

private:
    std::shared_ptr<RTPBundleTransport> transport;
};

std::unique_ptr<RtpBundleTransportFacade> new_rtp_bundle_transport(uint16_t port = 0);
