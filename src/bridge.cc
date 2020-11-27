#include "media-server-rs/include/bridge.h"
#include "media-server-rs/src/lib.rs.h"

#include "OpenSSL.h"

// This is from media-server, but it doesn't have an implementation.
// It should not actually ever be called.
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

rust::String dtls_connection_get_certificate_fingerprint(DtlsConnectionHash hash) {
    return DTLSConnection::GetCertificateFingerPrint(hash);
}

PropertiesFacade::PropertiesFacade():
    properties(std::make_unique<Properties>()) {}

PropertiesFacade::operator const Properties &() const {
    return *properties;
}

void PropertiesFacade::set_int(rust::Str key, int value) const {
    std::string keyString = std::string(key);
    properties->SetProperty(keyString.c_str(), value);
}

void PropertiesFacade::set_bool(rust::Str key, bool value) const {
    std::string keyString = std::string(key);
    properties->SetProperty(keyString.c_str(), value);
}

void PropertiesFacade::set_string(rust::Str key, rust::Str value) const {
    std::string keyString = std::string(key);
    std::string valueString = std::string(value);
    properties->SetProperty(keyString.c_str(), valueString.c_str());
}

std::unique_ptr<PropertiesFacade> new_properties() {
    return std::make_unique<PropertiesFacade>();
}

struct DtlsIceTransportListenerAdapter: DTLSICETransport::Listener {
    explicit DtlsIceTransportListenerAdapter(rust::Box<DtlsIceTransportListener> listener):
        listener(std::move(listener)) {};

    void onICETimeout() override {
        listener->on_ice_timeout();
    }

    void onDTLSStateChanged(const DtlsIceTransportDtlsState state) override {
        listener->on_dtls_state_changed(state);
    }

    void onRemoteICECandidateActivated(const std::string& ip, uint16_t port, uint32_t priority) override {
        listener->on_remote_ice_candidate_activated(ip, port, priority);
    }

    rust::Box<DtlsIceTransportListener> listener;
};

RTPBundleTransportConnectionFacade::RTPBundleTransportConnectionFacade(std::shared_ptr<RTPBundleTransport> transport, std::string username, RTPBundleTransport::Connection *connection):
    transport(std::move(transport)), username(std::move(username)), connection(connection), active_listener(nullptr) {}

RTPBundleTransportConnectionFacade::~RTPBundleTransportConnectionFacade() {
    connection->transport->SetListener(nullptr);
    active_listener = nullptr;

    transport->RemoveICETransport(username);
}

void RTPBundleTransportConnectionFacade::set_listener(rust::Box<DtlsIceTransportListener> listener) const {
    active_listener = std::make_unique<DtlsIceTransportListenerAdapter>(std::move(listener));
    connection->transport->SetListener(active_listener.get());
}

RtpBundleTransportFacade::RtpBundleTransportFacade():
    transport(std::make_shared<RTPBundleTransport>()) {}

unsigned short RtpBundleTransportFacade::init() const {
    return (unsigned short)transport->Init();
}

std::unique_ptr<RTPBundleTransportConnectionFacade> RtpBundleTransportFacade::add_ice_transport(rust::Str username, const PropertiesFacade &properties) const {
    std::string usernameString = std::string(username);
    auto connection = transport->AddICETransport(usernameString, properties);
    if (!connection) {
        // TODO: This should throw on failure.
        return nullptr;
    }

    return std::make_unique<RTPBundleTransportConnectionFacade>(transport, usernameString, connection);
}

std::unique_ptr<RtpBundleTransportFacade> new_rtp_bundle_transport() {
    return std::make_unique<RtpBundleTransportFacade>();
}
