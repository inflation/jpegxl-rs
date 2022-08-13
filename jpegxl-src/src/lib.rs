use std::{
    env,
    path::{Path, PathBuf},
};

fn source_dir() -> PathBuf {
    env::var("DEP_JXL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("libjxl"))
}

pub fn build() {
    use cmake::Config;

    let source = source_dir();
    if !source.exists() {
        panic!("libjxl source not found at: {}", source.display());
    }

    env::set_var("CMAKE_BUILD_PARALLEL_LEVEL", format!("{}", num_cpus::get()));

    let mut config = Config::new(&source);
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
        .define("JPEGXL_ENABLE_OPENEXR", "OFF");

    let mut prefix = config.build();
    println!("cargo:rustc-link-lib=static=jxl");
    println!("cargo:rustc-link-lib=static=jxl_threads");

    println!("cargo:rustc-link-lib=static=hwy");
    prefix.push("lib");
    println!("cargo:rustc-link-search=native={}", prefix.display());
    prefix.pop();

    prefix.push("build");
    prefix.push("third_party");
    println!("cargo:rustc-link-search=native={}", prefix.display());

    println!("cargo:rustc-link-lib=static=brotlicommon-static");
    println!("cargo:rustc-link-lib=static=brotlidec-static");
    println!("cargo:rustc-link-lib=static=brotlienc-static");
    prefix.push("brotli");
    println!("cargo:rustc-link-search=native={}", prefix.display());

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
    println!("cargo:rustc-link-lib=c++");
    #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "freebsd")))]
    println!("cargo:rustc-link-lib=stdc++");
}
