fn main() {
    #[cfg(all(not(feature = "vendored"), not(feature = "docs")))]
    {
        use std::env;
        const VERSION: &str = "0.7rc";

        if let Ok(path) = env::var("DEP_JXL_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=jxl");
            println!("cargo:rustc-link-lib=jxl_threads");
        } else {
            pkg_config::Config::new()
                .atleast_version(VERSION)
                .probe("libjxl")
                .expect(&format!("Cannot find `libjxl` with version >= {}", VERSION));
            pkg_config::Config::new()
                .atleast_version(VERSION)
                .probe("libjxl_threads")
                .expect(&format!("Cannot find `libjxl` with version >= {}", VERSION));
        }
    }

    #[cfg(feature = "vendored")]
    jpegxl_src::build()
}
