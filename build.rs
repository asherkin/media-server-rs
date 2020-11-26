use std::path::PathBuf;

fn main() {
    let openssl = pkg_config::probe_library("openssl").unwrap();
    let openssl_include_paths: Vec<_> = openssl.include_paths.iter().map(PathBuf::as_path).collect();
    
    let srtp_include_paths = vec![
        "media-server-node/external/srtp/config",
        "media-server-node/external/srtp/lib/include",
        "media-server-node/external/srtp/lib/crypto/include",
    ];

    let srtp_files = vec![
        "media-server-node/external/srtp/lib/srtp/ekt.c",
        "media-server-node/external/srtp/lib/srtp/srtp.c",
        "media-server-node/external/srtp/lib/crypto/cipher/aes.c",
        "media-server-node/external/srtp/lib/crypto/cipher/cipher.c",
        "media-server-node/external/srtp/lib/crypto/cipher/null_cipher.c",
        "media-server-node/external/srtp/lib/crypto/hash/auth.c",
        "media-server-node/external/srtp/lib/crypto/hash/null_auth.c",
        "media-server-node/external/srtp/lib/crypto/kernel/alloc.c",
        "media-server-node/external/srtp/lib/crypto/kernel/crypto_kernel.c",
        "media-server-node/external/srtp/lib/crypto/kernel/err.c",
        "media-server-node/external/srtp/lib/crypto/kernel/key.c",
        "media-server-node/external/srtp/lib/crypto/math/datatypes.c",
        "media-server-node/external/srtp/lib/crypto/math/stat.c",
        "media-server-node/external/srtp/lib/crypto/replay/rdb.c",
        "media-server-node/external/srtp/lib/crypto/replay/rdbx.c",
        "media-server-node/external/srtp/lib/crypto/replay/ut_sim.c",
        "media-server-node/external/srtp/lib/crypto/cipher/aes_gcm_ossl.c",
        "media-server-node/external/srtp/lib/crypto/cipher/aes_icm_ossl.c",
        "media-server-node/external/srtp/lib/crypto/hash/hmac_ossl.c",
    ];

    cc::Build::new()
        .warnings(false)
        .flag_if_supported("-march=native")
        .define("HAVE_CONFIG_H", None)
        .define("HAVE_STDLIB_H", None)
        .define("HAVE_STRING_H", None)
        .define("TESTAPP_SOURCE", None)
        .define("OPENSSL", None)
        .define("HAVE_INT16_T", None)
        .define("HAVE_INT32_T", None)
        .define("HAVE_INT8_T", None)
        .define("HAVE_UINT16_T", None)
        .define("HAVE_UINT32_T", None)
        .define("HAVE_UINT64_T", None)
        .define("HAVE_UINT8_T", None)
        .define("HAVE_STDINT_H", None)
        .define("HAVE_INTTYPES_H", None)
        .define("HAVE_NETINET_IN_H", None)
        .define("HAVE_ARPA_INET_H", None)
        .define("HAVE_UNISTD_H", None)
        .define("CPU_CISC", None)
        .define("HAVE_X86", None)
        .files(&srtp_files)
        .includes(&openssl_include_paths)
        .includes(&srtp_include_paths)
        .compile("srtp");

    let media_server_include_paths = vec![
        "media-server-node/media-server/include",
        "media-server-node/media-server/src",
        "media-server-node/media-server/ext/crc32c/include",
        "media-server-node/media-server/ext/libdatachannels/src",
        "media-server-node/media-server/ext/libdatachannels/src/internal",
        "media-server-node/external/mp4v2/lib/include",
        "media-server-node/external/mp4v2/config/include",
        "media-server-node/external/srtp/include",
        "media-server-node/media-server/ext/crc32c/config/Darwin-i386",
    ];

    let media_server_files = vec![
        "media-server-node/media-server/ext/crc32c/src/crc32c.cc",
        "media-server-node/media-server/ext/crc32c/src/crc32c_portable.cc",
        "media-server-node/media-server/ext/crc32c/src/crc32c_sse42.cc",
        // "media-server-node/media-server/ext/crc32c/src/crc32c_arm64.cc",
        "media-server-node/media-server/ext/libdatachannels/src/Datachannels.cpp",
        "media-server-node/media-server/src/ActiveSpeakerDetector.cpp",
        "media-server-node/media-server/src/EventLoop.cpp",
        "media-server-node/media-server/src/RTPBundleTransport.cpp",
        "media-server-node/media-server/src/DTLSICETransport.cpp",
        "media-server-node/media-server/src/VideoLayerSelector.cpp",
        "media-server-node/media-server/src/opus/opusdepacketizer.cpp",
        "media-server-node/media-server/src/h264/h264depacketizer.cpp",
        "media-server-node/media-server/src/vp8/vp8depacketizer.cpp",
        "media-server-node/media-server/src/h264/H264LayerSelector.cpp",
        "media-server-node/media-server/src/vp8/VP8LayerSelector.cpp",
        "media-server-node/media-server/src/vp9/VP9PayloadDescription.cpp",
        "media-server-node/media-server/src/vp9/VP9LayerSelector.cpp",
        "media-server-node/media-server/src/vp9/VP9Depacketizer.cpp",
        "media-server-node/media-server/src/SRTPSession.cpp",
        "media-server-node/media-server/src/dtls.cpp",
        "media-server-node/media-server/src/CPUMonitor.cpp",
        "media-server-node/media-server/src/OpenSSL.cpp",
        "media-server-node/media-server/src/RTPTransport.cpp",
        "media-server-node/media-server/src/httpparser.cpp",
        "media-server-node/media-server/src/stunmessage.cpp",
        "media-server-node/media-server/src/crc32calc.cpp",
        "media-server-node/media-server/src/http.cpp",
        "media-server-node/media-server/src/avcdescriptor.cpp",
        "media-server-node/media-server/src/utf8.cpp",
        "media-server-node/media-server/src/DependencyDescriptorLayerSelector.cpp",
        "media-server-node/media-server/src/rtp/DependencyDescriptor.cpp",
        "media-server-node/media-server/src/rtp/LayerInfo.cpp",
        "media-server-node/media-server/src/rtp/RTCPCommonHeader.cpp",
        "media-server-node/media-server/src/rtp/RTPHeader.cpp",
        "media-server-node/media-server/src/rtp/RTPHeaderExtension.cpp",
        "media-server-node/media-server/src/rtp/RTCPApp.cpp",
        "media-server-node/media-server/src/rtp/RTCPExtendedJitterReport.cpp",
        "media-server-node/media-server/src/rtp/RTCPPacket.cpp",
        // "media-server-node/media-server/src/rtp/RTCPReport.cpp",
        "media-server-node/media-server/src/rtp/RTCPSenderReport.cpp",
        // "media-server-node/media-server/src/rtp/RTPMap.cpp",
        "media-server-node/media-server/src/rtp/RTCPBye.cpp",
        "media-server-node/media-server/src/rtp/RTCPFullIntraRequest.cpp",
        "media-server-node/media-server/src/rtp/RTCPPayloadFeedback.cpp",
        "media-server-node/media-server/src/rtp/RTCPRTPFeedback.cpp",
        "media-server-node/media-server/src/rtp/RTPDepacketizer.cpp",
        "media-server-node/media-server/src/rtp/RTPPacket.cpp",
        "media-server-node/media-server/src/rtp/RTPPayload.cpp",
        "media-server-node/media-server/src/rtp/RTCPCompoundPacket.cpp",
        "media-server-node/media-server/src/rtp/RTCPNACK.cpp",
        "media-server-node/media-server/src/rtp/RTCPReceiverReport.cpp",
        "media-server-node/media-server/src/rtp/RTCPSDES.cpp",
        // "media-server-node/media-server/src/rtp/RTPPacketSched.cpp",
        "media-server-node/media-server/src/rtp/RTPStreamTransponder.cpp",
        "media-server-node/media-server/src/rtp/RTPLostPackets.cpp",
        "media-server-node/media-server/src/rtp/RTPSource.cpp",
        "media-server-node/media-server/src/rtp/RTPIncomingMediaStreamMultiplexer.cpp",
        "media-server-node/media-server/src/rtp/RTPIncomingMediaStreamDepacketizer.cpp",
        "media-server-node/media-server/src/rtp/RTPIncomingSource.cpp",
        "media-server-node/media-server/src/rtp/RTPIncomingSourceGroup.cpp",
        "media-server-node/media-server/src/rtp/RTPOutgoingSource.cpp",
        "media-server-node/media-server/src/rtp/RTPOutgoingSourceGroup.cpp",
        "media-server-node/media-server/src/mp4recorder.cpp",
        "media-server-node/media-server/src/mp4streamer.cpp",
        "media-server-node/media-server/src/rtpsession.cpp",
        "media-server-node/media-server/src/PCAPFile.cpp",
        "media-server-node/media-server/src/PCAPReader.cpp",
        "media-server-node/media-server/src/PCAPTransportEmulator.cpp",
        "media-server-node/media-server/src/remoteratecontrol.cpp",
        "media-server-node/media-server/src/remoterateestimator.cpp",
        "media-server-node/media-server/src/SendSideBandwidthEstimation.cpp",
    ];

    cc::Build::new()
        .cpp(true)
        .warnings(false)
        .cpp_link_stdlib(None)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-march=native")
        .flag_if_supported("-Wno-switch")
        .flag_if_supported("-Wno-format")
        .files(&media_server_files)
        .includes(&openssl_include_paths)
        .includes(&media_server_include_paths)
        .compile("media-server");

    cxx_build::bridge("src/lib.rs")
        .warnings(false)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-march=native")
        .file("src/bridge.cc")
        .includes(&openssl_include_paths)
        .includes(&media_server_include_paths)
        .compile("media-server-rs");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/bridge.cc");
    println!("cargo:rerun-if-changed=include/bridge.h");
}