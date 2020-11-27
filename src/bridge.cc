#include "media-server-rs/include/bridge.h"

#include "log.h"
#include "OpenSSL.h"

#include "media-server-rs/src/lib.rs.h"

void EvenSource::SendEvent(const char *type, const char *msg, ...) {
    Debug("-EvenSource::SendEvent(%s, %s, ...)", type, msg);
}

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

rust::String dtls_connection_get_certificate_fingerprint(DTLSConnectionHash hash) {
    return DTLSConnection::GetCertificateFingerPrint(hash);
}

std::unique_ptr<Properties> new_properties() {
    return std::make_unique<Properties>();
}

void properties_set_int(const std::unique_ptr<Properties> &properties, rust::Str key, int value) {
    std::string keyString = std::string(key);
    properties->SetProperty(keyString.c_str(), value);
}

void properties_set_bool(const std::unique_ptr<Properties> &properties, rust::Str key, bool value) {
    std::string keyString = std::string(key);
    properties->SetProperty(keyString.c_str(), value);
}

void properties_set_string(const std::unique_ptr<Properties> &properties, rust::Str key, rust::Str value) {
    std::string keyString = std::string(key);
    std::string valueString = std::string(value);
    properties->SetProperty(keyString.c_str(), valueString.c_str());
}

std::unique_ptr<RTPBundleTransport> new_rtp_bundle_transport() {
    return std::make_unique<RTPBundleTransport>();
}

std::unique_ptr<RTPBundleTransportConnectionWrapper> rtp_bundle_transport_add_ice_transport(const std::unique_ptr<RTPBundleTransport> &transport, rust::Str username, const Properties &properties) {
    auto connection = transport->AddICETransport(std::string(username), properties);
    if (!connection) {
        return nullptr;
    }

    return std::make_unique<RTPBundleTransportConnectionWrapper>(connection);
}

void rtp_bundle_transport_remove_ice_transport(const std::unique_ptr<RTPBundleTransport> &transport, rust::Str username) {
    transport->RemoveICETransport(std::string(username));
}

struct DTLSICETransportListenerWrapper: DTLSICETransport::Listener {
    DTLSICETransportListenerWrapper(DtlsIceTransportListener &listener):
        listener(listener) {};

    virtual void onICETimeout() override {
        listener.on_ice_timeout();
    }

    virtual void onDTLSStateChanged(const DTLSICETransportDTLSState state) override {
        listener.on_dtls_state_changed(state);
    }

    virtual void onRemoteICECandidateActivated(const std::string& ip, uint16_t port, uint32_t priority) override {
        listener.on_remote_ice_candidate_activated(ip, port, priority);
    }

    DtlsIceTransportListener &listener;
};

void rtp_bundle_transport_connection_set_listener(const std::unique_ptr<RTPBundleTransportConnectionWrapper> &wrapper, DtlsIceTransportListener &listener) {
    // TODO: We're leaking this.
    wrapper->connection->transport->SetListener(new DTLSICETransportListenerWrapper(listener));
}
