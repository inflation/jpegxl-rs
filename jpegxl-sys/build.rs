use std::env;

#[cfg(feature = "vendored")]
use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
    process::Output,
};

const VERSION: &str = "0.6.1";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_jpegxl()?;

    Ok(())
}

fn setup_jpegxl() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "vendored"))]
    {
        if let Ok(path) = env::var("DEP_JXL_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=jxl");
            #[cfg(feature = "threads")]
            println!("cargo:rustc-link-lib=jxl_threads");
            Ok(())
        } else {
            pkg_config::Config::new()
                .atleast_version(VERSION)
                .probe("libjxl")?;
            #[cfg(feature = "threads")]
            pkg_config::Config::new()
                .atleast_version(VERSION)
                .probe("libjxl_threads")?;
            Ok(())
        }
    }

    #[cfg(feature = "vendored")]
    build()
}

#[cfg(feature = "vendored")]
fn check_status(msg: &'static str) -> impl Fn(Output) -> Result<(), Error> {
    move |e| {
        e.status.success().then(|| ()).ok_or_else(|| {
            Error::new(
                ErrorKind::Other,
                format!("{}, stderr: {}", msg, String::from_utf8_lossy(&e.stderr)),
            )
        })
    }
}

#[cfg(feature = "vendored")]
fn build() -> Result<(), Box<dyn std::error::Error>> {
    use cmake::Config;
    use std::process::Command;

    let source: PathBuf = env::var("DEP_JXL_PATH")
        .unwrap_or_else(|_| {
            format!(
                "{}/{}",
                &env::var("OUT_DIR").expect("Failed to get OUT_DIR. Not running under cargo?"),
                "libjxl"
            )
        })
        .into();
    let source_str = source.to_str().ok_or("Source path is invalid UTF-8")?;

    if !source.exists() {
        Command::new("git")
            .args(&[
                "clone",
                "--depth=1",
                &format!("--branch=v{}", VERSION),
                "https://github.com/libjxl/libjxl.git",
                source_str,
            ])
            .output()
            .and_then(check_status("Failed to clone libjxl"))?;
    }
    Command::new("git")
        .args(&["-C", source_str, "submodule", "init"])
        .output()
        .and_then(check_status("Failed to init submodule!"))?;
    Command::new("git")
        .args(&["-C", source_str, "submodule", "update", "--depth=1"])
        .output()
        .and_then(check_status("Failed to update submodule!"))?;

    env::set_var("CMAKE_BUILD_PARALLEL_LEVEL", format!("{}", num_cpus::get()));

    let mut config = Config::new(&source);
    config
        .define("JPEGXL_ENABLE_TOOLS", "OFF")
        .define("JPEGXL_ENABLE_MANPAGES", "OFF")
        .define("JPEGXL_ENABLE_BENCHMARK", "OFF")
        .define("JPEGXL_ENABLE_EXAMPLES", "OFF")
        .define("JPEGXL_ENABLE_JNI", "OFF")
        .define("JPEGXL_ENABLE_SJPEG", "OFF")
        .define("JPEGXL_ENABLE_OPENEXR", "OFF")
        .define("JPEGXL_STATIC", "ON");

    let mut prefix = config.build();
    println!("cargo:rustc-link-lib=static=jxl");

    #[cfg(feature = "threads")]
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

    #[cfg(feature = "threads")]
    {
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
        println!("cargo:rustc-link-lib=c++");
        #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "freebsd")))]
        println!("cargo:rustc-link-lib=stdc++");
    }

    Ok(())
}
