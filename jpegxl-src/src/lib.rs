#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::{
    env,
    path::{Path, PathBuf},
};

fn source_dir() -> PathBuf {
    env::var("DEP_JXL_PATH").map_or_else(
        |_| Path::new(env!("CARGO_MANIFEST_DIR")).join("libjxl"),
        PathBuf::from,
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn build() {
    let source = source_dir();

    if let Ok(p) = std::thread::available_parallelism() {
        env::set_var("CMAKE_BUILD_PARALLEL_LEVEL", format!("{}", p))
    }

    let mut config = cmake::Config::new(source);
    config
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("JPEGXL_ENABLE_TOOLS", "OFF")
        .define("JPEGXL_ENABLE_DOXYGEN", "OFF")
        .define("JPEGXL_ENABLE_MANPAGES", "OFF")
        .define("JPEGXL_ENABLE_BENCHMARK", "OFF")
        .define("JPEGXL_ENABLE_EXAMPLES", "OFF")
        .define("JPEGXL_ENABLE_JNI", "OFF")
        .define("JPEGXL_ENABLE_SJPEG", "OFF")
        .define("JPEGXL_ENABLE_OPENEXR", "OFF")
        .define("JPEGXL_ENABLE_JPEGLI", "OFF")
        .define("CMAKE_INSTALL_LIBDIR", "lib");

    let mut prefix = config.build();
    prefix.push("lib");
    println!("cargo:rustc-link-search=native={}", prefix.display());

    println!("cargo:rustc-link-lib=static=jxl");
    println!("cargo:rustc-link-lib=static=jxl_cms");
    println!("cargo:rustc-link-lib=static=jxl_threads");

    println!("cargo:rustc-link-lib=static=hwy");
    println!("cargo:rustc-link-lib=static=brotlicommon");
    println!("cargo:rustc-link-lib=static=brotlidec");
    println!("cargo:rustc-link-lib=static=brotlienc");

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
    println!("cargo:rustc-link-lib=c++");
    #[cfg(not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_env = "msvc"
    )))]
    println!("cargo:rustc-link-lib=stdc++");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_source_dir() {
        let mut path = source_dir();
        assert!(path.is_dir());

        path.push("lib/include/jxl/codestream_header.h");
        assert!(path.exists());
    }
}
