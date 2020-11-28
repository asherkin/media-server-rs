cmake_minimum_required(VERSION 3.13)
project(media-server)
set(CMAKE_CXX_STANDARD 17)

set(MEDIA_SERVER_INCLUDE media-server-node/media-server/include media-server-node/media-server/src media-server-node/media-server/ext/crc32c/include media-server-node/media-server/ext/libdatachannels/src media-server-node/media-server/ext/libdatachannels/src/internal media-server-node/external/mp4v2/lib/include media-server-node/external/mp4v2/config/include media-server-node/external/srtp/include media-server-node/media-server/ext/crc32c/config/Darwin-i386)
include_directories(target/cxxbridge ${MEDIA_SERVER_INCLUDE})

# This CMakeLists isn't used by the Rust build, it just pulls all the C++ files together to get IDE support.
add_executable(dummy include/bridge.h src/bridge/bridge.cc target/cxxbridge/media-server/src/lib.rs.h target/cxxbridge/media-server/src/lib.rs.cc)