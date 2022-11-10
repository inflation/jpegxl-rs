fn main() {
    #[cfg(all(not(feature = "vendored"), not(feature = "docs")))]
    {
        use std::env;
        let version = env!("CARGO_PKG_VERSION")
            .split('+')
            .nth(1)
            .and_then(|s| s.split('-').nth(1))
            .unwrap();

        if let Ok(path) = env::var("DEP_JXL_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=jxl");
            println!("cargo:rustc-link-lib=jxl_threads");
        } else {
            pkg_config::Config::new()
                .atleast_version(version)
                .probe("libjxl")
                .unwrap_or_else(|_| panic!("Cannot find `libjxl` with version >= {}", version));
            #[cfg(feature = "threads")]
            pkg_config::Config::new()
                .atleast_version(version)
                .probe("libjxl_threads")
                .unwrap_or_else(|_| {
                    panic!("Cannot find `libjxl_threads` with version >= {}", version)
                });
        }
    }

    #[cfg(feature = "vendored")]
    jpegxl_src::build()
}
