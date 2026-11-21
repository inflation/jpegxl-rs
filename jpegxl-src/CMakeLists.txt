cmake_minimum_required(VERSION 3.16...3.27)
project(jpegxl_src LANGUAGES C CXX)

add_subdirectory("${LIBJXL_SOURCE_DIR}" libjxl)

# These public C APIs use internal codec symbols and are omitted from upstream's core archive.
target_sources(jxl_enc-obj PRIVATE
    "${LIBJXL_SOURCE_DIR}/lib/extras/compressed_icc.cc"
    "${LIBJXL_SOURCE_DIR}/lib/extras/gain_map.cc"
)
