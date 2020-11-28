#include "media-server/include/bridge.h"
#include "media-server/src/bridge/mod.rs.h"

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
    if (!OpenSSL::ClassInit()) {
        throw std::runtime_error("openssl initialization failed");
    }
}

void dtls_connection_initialize() {
    if (DTLSConnection::Initialize() == 0) {
        throw std::runtime_error("dtls initialization failed");
    }
}

rust::String dtls_connection_get_certificate_fingerprint(DtlsConnectionHash hash) {
    auto fingerprint = DTLSConnection::GetCertificateFingerPrint(hash);
    if (fingerprint.empty()) {
        throw std::runtime_error("no certificate fingerprint for hash - dtls not initialized?");
    }

    return fingerprint;
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

struct DtlsIceTransportListenerCxxAdapter: DTLSICETransport::Listener {
    explicit DtlsIceTransportListenerCxxAdapter(rust::Box<DtlsIceTransportListenerRustAdapter> listener):
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

    rust::Box<DtlsIceTransportListenerRustAdapter> listener;
};

RTPBundleTransportConnectionFacade::RTPBundleTransportConnectionFacade(std::shared_ptr<RTPBundleTransport> transport, std::string username, RTPBundleTransport::Connection *connection):
    transport(std::move(transport)), username(std::move(username)), connection(connection), active_listener(nullptr) {}

RTPBundleTransportConnectionFacade::~RTPBundleTransportConnectionFacade() {
    connection->transport->SetListener(nullptr);
    active_listener = nullptr;

    transport->RemoveICETransport(username);
}

void RTPBundleTransportConnectionFacade::set_listener(rust::Box<DtlsIceTransportListenerRustAdapter> listener) const {
    active_listener = std::make_unique<DtlsIceTransportListenerCxxAdapter>(std::move(listener));
    connection->transport->SetListener(active_listener.get());
}

void RTPBundleTransportConnectionFacade::add_remote_candidate(rust::Str ip, uint16_t port) const {
    std::string ipString = std::string(ip);
    transport->AddRemoteCandidate(username, ipString.c_str(), port);
}

RtpBundleTransportFacade::RtpBundleTransportFacade(uint16_t port):
    transport(std::make_shared<RTPBundleTransport>()) {
    if (transport->Init(port) == 0) {
        throw std::runtime_error("failed to open socket");
    }
}

uint16_t RtpBundleTransportFacade::get_local_port() const {
    return (uint16_t)transport->GetLocalPort();
}

std::unique_ptr<RTPBundleTransportConnectionFacade> RtpBundleTransportFacade::add_ice_transport(rust::Str username, const PropertiesFacade &properties) const {
    std::string usernameString = std::string(username);

    auto connection = transport->AddICETransport(usernameString, properties);
    if (!connection) {
        throw std::runtime_error("ice transport creation failed");
    }

    return std::make_unique<RTPBundleTransportConnectionFacade>(transport, usernameString, connection);
}

std::unique_ptr<RtpBundleTransportFacade> new_rtp_bundle_transport(uint16_t port) {
    return std::make_unique<RtpBundleTransportFacade>(port);
}
