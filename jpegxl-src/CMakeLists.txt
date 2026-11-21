cmake_minimum_required(VERSION 3.16...3.27)

# Cargo supplies this option only for an MSVC target. Set the CRT before
# compiler detection so CMake's checks and the vendored targets agree.
if(DEFINED JPEGXL_RS_STATIC_CRT)
    if(JPEGXL_RS_STATIC_CRT)
        set(runtime_library MultiThreaded)
    else()
        set(runtime_library MultiThreadedDLL)
    endif()
    set(CMAKE_MSVC_RUNTIME_LIBRARY "${runtime_library}" CACHE STRING "Cargo target CRT" FORCE)
endif()

project(jpegxl_src LANGUAGES C CXX)

foreach(option IN ITEMS
        BUILD_TESTING BUILD_SHARED_LIBS JPEGXL_ENABLE_TOOLS JPEGXL_ENABLE_DOXYGEN
        JPEGXL_ENABLE_MANPAGES JPEGXL_ENABLE_BENCHMARK JPEGXL_ENABLE_EXAMPLES
        JPEGXL_ENABLE_JNI JPEGXL_ENABLE_SJPEG JPEGXL_ENABLE_OPENEXR JPEGXL_BUNDLE_LIBPNG)
    set(${option} OFF CACHE BOOL "Disabled for the vendored Rust library" FORCE)
endforeach()

if(CMAKE_GENERATOR MATCHES "^Visual Studio")
    list(APPEND CMAKE_VS_GLOBALS UseMultiToolTask=true EnforceProcessCountAcrossBuilds=true)
endif()

if(JPEGXL_RS_SANITIZER)
    if(WIN32 AND JPEGXL_RS_SANITIZER STREQUAL "thread")
        message(FATAL_ERROR "Thread sanitizer is not supported on Windows")
    endif()
    string(TOUPPER "${JPEGXL_RS_SANITIZER}" sanitizer_define)
    add_compile_definitions("${sanitizer_define}_SANITIZER")
    add_compile_options(-g "-fsanitize=${JPEGXL_RS_SANITIZER}")
    set(JPEGXL_ENABLE_TCMALLOC OFF CACHE BOOL "Incompatible with sanitizers" FORCE)
    if(MSVC)
        set(asan_arch "${JPEGXL_RS_TARGET_ARCH}")
        if(asan_arch STREQUAL "x86")
            set(asan_arch i386)
        endif()
        string(APPEND CMAKE_EXE_LINKER_FLAGS
            " clang_rt.asan_dynamic-${asan_arch}.lib clang_rt.asan_dynamic_runtime_thunk-${asan_arch}.lib")
    else()
        add_link_options("-fsanitize=${JPEGXL_RS_SANITIZER}")
    endif()
endif()

add_subdirectory("${LIBJXL_SOURCE_DIR}" libjxl)

if(JPEGXL_RS_SANITIZER STREQUAL "address" AND NOT MSVC)
    # ASan registers each SIMD implementation's lookup tables as separate globals.
    target_compile_options(jxl_enc-obj PRIVATE -fno-merge-all-constants)
endif()

# These public C APIs use internal codec symbols and are omitted from upstream's core archive.
# FIXME: On libjxl upgrades, remove this block if libjxl exports these APIs directly,
# or link jxl_extras_codec if upstream makes it available with tools and tests disabled.
target_sources(jxl_enc-obj PRIVATE
    "${LIBJXL_SOURCE_DIR}/lib/extras/compressed_icc.cc"
    "${LIBJXL_SOURCE_DIR}/lib/extras/gain_map.cc"
)
