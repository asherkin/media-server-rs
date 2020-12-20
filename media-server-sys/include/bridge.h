#pragma once
#include "rust/cxx.h"

#include "DTLSICETransport.h"
#include "RTPBundleTransport.h"
#include "rtp/RTPStreamTransponder.h"

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

void rtp_transport_set_port_range(uint16_t min, uint16_t max);

struct PropertiesFacade {
    operator const Properties &() const;
    void set_int(rust::Str key, int value);
    void set_bool(rust::Str key, bool value);
    void set_string(rust::Str key, rust::Str value);

private:
    Properties properties;
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

struct RtpStreamTransponderFacade;

struct OwnedRtpIncomingSourceGroup {
    OwnedRtpIncomingSourceGroup(std::shared_ptr<OwnedRtpBundleTransportConnection> connection, std::unique_ptr<RTPIncomingSourceGroup> source_group);
    ~OwnedRtpIncomingSourceGroup();
    RTPIncomingSourceGroup *operator->();

private:
    std::unique_ptr<RTPIncomingSourceGroup> source_group;
    std::shared_ptr<OwnedRtpBundleTransportConnection> connection;

    friend struct RtpStreamTransponderFacade;
};

struct RtpIncomingSourceGroupFacade {
    RtpIncomingSourceGroupFacade(std::shared_ptr<OwnedRtpIncomingSourceGroup> source_group);

private:
    std::shared_ptr<OwnedRtpIncomingSourceGroup> source_group;

    friend struct RtpStreamTransponderFacade;
};

struct OwnedRtpOutgoingSourceGroup {
    OwnedRtpOutgoingSourceGroup(std::shared_ptr<OwnedRtpBundleTransportConnection> connection, std::unique_ptr<RTPOutgoingSourceGroup> source_group);
    ~OwnedRtpOutgoingSourceGroup();
    RTPOutgoingSourceGroup *operator->();

private:
    std::unique_ptr<RTPOutgoingSourceGroup> source_group;
    std::shared_ptr<OwnedRtpBundleTransportConnection> connection;

    friend struct RtpStreamTransponderFacade;
};

struct RtpOutgoingSourceGroupFacade {
    RtpOutgoingSourceGroupFacade(std::shared_ptr<OwnedRtpOutgoingSourceGroup> source_group);

    std::unique_ptr<RtpStreamTransponderFacade> add_transponder();

private:
    std::shared_ptr<OwnedRtpOutgoingSourceGroup> source_group;

    friend struct RtpStreamTransponderFacade;
};

struct RtpStreamTransponderFacade {
    explicit RtpStreamTransponderFacade(RtpOutgoingSourceGroupFacade &outgoing);
    void set_incoming(RtpIncomingSourceGroupFacade &new_incoming);

private:
    std::shared_ptr<OwnedRtpIncomingSourceGroup> incoming;
    std::shared_ptr<OwnedRtpOutgoingSourceGroup> outgoing;
    std::unique_ptr<RTPStreamTransponder> transponder;
};

struct RtpBundleTransportConnectionFacade {
    RtpBundleTransportConnectionFacade(std::shared_ptr<RTPBundleTransport> transport, std::shared_ptr<OwnedRtpBundleTransportConnection> connection);
    ~RtpBundleTransportConnectionFacade();
    void set_listener(rust::Box<DtlsIceTransportListenerRustAdapter> listener);
    void set_remote_properties(const PropertiesFacade &properties);
    void set_local_properties(const PropertiesFacade &properties);
    std::unique_ptr<RtpIncomingSourceGroupFacade> add_incoming_source_group(MediaFrameType type, rust::Str mid, rust::Str rid, uint32_t mediaSsrc, uint32_t rtxSsrc);
    std::unique_ptr<RtpOutgoingSourceGroupFacade> add_outgoing_source_group(MediaFrameType type, rust::Str mid, uint32_t mediaSsrc, uint32_t rtxSsrc);
    void add_remote_candidate(rust::Str ip, uint16_t port);

private:
    std::shared_ptr<RTPBundleTransport> transport;
    std::shared_ptr<OwnedRtpBundleTransportConnection> connection;
    std::unique_ptr<DtlsIceTransportListenerCxxAdapter> active_listener;
};

struct RtpBundleTransportFacade {
    RtpBundleTransportFacade(uint16_t port = 0);
    uint16_t get_local_port() const;
    std::unique_ptr<RtpBundleTransportConnectionFacade> add_ice_transport(rust::Str username, const PropertiesFacade &properties);

private:
    std::shared_ptr<RTPBundleTransport> transport;
};

std::unique_ptr<RtpBundleTransportFacade> new_rtp_bundle_transport(uint16_t port = 0);
