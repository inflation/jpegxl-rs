cmake_minimum_required(VERSION 3.16...3.27)
project(jpegxl_src LANGUAGES C CXX)

add_subdirectory("${LIBJXL_SOURCE_DIR}" libjxl)

if("$ENV{SANITIZER}" STREQUAL "asan" AND NOT MSVC)
    # ASan registers each SIMD implementation's lookup tables as separate globals.
    target_compile_options(jxl_enc-obj PRIVATE -fno-merge-all-constants)
endif()

# These public C APIs use internal codec symbols and are omitted from upstream's core archive.
target_sources(jxl_enc-obj PRIVATE
    "${LIBJXL_SOURCE_DIR}/lib/extras/compressed_icc.cc"
    "${LIBJXL_SOURCE_DIR}/lib/extras/gain_map.cc"
)
