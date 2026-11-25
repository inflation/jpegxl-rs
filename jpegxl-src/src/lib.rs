/*
 * This file is part of jpegxl-rs.
 *
 * jpegxl-rs is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * jpegxl-rs is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
 */

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

fn cpp_runtime(os: &str, vendor: &str, environment: &str, abi: &str) -> Option<&'static str> {
    match (os, vendor, environment, abi) {
        (_, "apple", _, _) | ("freebsd", _, _, _) | ("windows", _, "gnu", "llvm") => Some("c++"),
        ("linux", _, _, _) | ("windows", _, "gnu", _) => Some("stdc++"),
        _ => None,
    }
}

struct MsvcOptions {
    static_crt: bool,
    clang_cl_toolset: bool,
}

impl MsvcOptions {
    fn from_cargo(var: impl Fn(&str) -> Option<String>) -> Self {
        let target = var("TARGET").expect("Cargo target triple");
        let build_kind = if var("HOST").as_ref() == Some(&target) {
            "HOST"
        } else {
            "TARGET"
        };
        let generator = [
            format!("CMAKE_GENERATOR_{target}"),
            format!("CMAKE_GENERATOR_{}", target.replace('-', "_")),
            format!("{build_kind}_CMAKE_GENERATOR"),
            "CMAKE_GENERATOR".to_owned(),
        ]
        .iter()
        .find_map(|key| var(key));
        let target_features = var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
        Self {
            static_crt: target_features
                .split(',')
                .any(|feature| feature == "crt-static"),
            clang_cl_toolset: generator
                .as_deref()
                .map_or(cfg!(windows), |name| name.starts_with("Visual Studio")),
        }
    }
}

/// The `CMAKE_BUILD_PARALLEL_LEVEL` to set for the libjxl build, or `None` to keep the one of the
/// environment. Without one, the build runs as many jobs as Cargo grants the build script
/// (`NUM_JOBS`), and as many as there are processors outside Cargo.
fn build_parallel_level(var: impl Fn(&str) -> Option<String>) -> Option<String> {
    if var("CMAKE_BUILD_PARALLEL_LEVEL").is_some() {
        return None;
    }
    var("NUM_JOBS").or_else(|| {
        std::thread::available_parallelism()
            .ok()
            .map(|p| p.to_string())
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
/// Builds the JPEG XL library.
///
/// # Panics
///
/// This function will panic if the source directory does not exist or is not a directory.
pub fn build() {
    let source = source_dir();
    assert!(
        source.is_dir(),
        "Source directory {} does not exist",
        source.display()
    );

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = env::current_dir()
        .expect("build script working directory")
        .join(source);
    println!("cargo:rerun-if-env-changed=DEP_JXL_PATH");
    println!(
        "cargo:rerun-if-changed={}",
        root.join("CMakeLists.txt").display()
    );
    println!("cargo:rerun-if-changed={}", source.display());
    let mut config = cmake::Config::new(root);
    config.define("LIBJXL_SOURCE_DIR", source);

    println!("cargo:rerun-if-env-changed=CMAKE_BUILD_PARALLEL_LEVEL");
    if let Some(level) = build_parallel_level(|key| env::var(key).ok()) {
        config.env("CMAKE_BUILD_PARALLEL_LEVEL", level);
    }

    let sanitizer = if env::var_os("CARGO_CFG_ASAN").is_some() {
        "address"
    } else if env::var_os("CARGO_CFG_TSAN").is_some() {
        "thread"
    } else {
        ""
    };
    config.define("JPEGXL_RS_SANITIZER", sanitizer);

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").expect("Cargo target environment");
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").expect("Cargo target vendor");
    if target_env == "msvc" {
        let options = MsvcOptions::from_cargo(|key| env::var(key).ok());
        if sanitizer == "address" {
            config.define(
                "JPEGXL_RS_TARGET_ARCH",
                env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo target architecture"),
            );
        }
        if options.clang_cl_toolset {
            config.generator_toolset("ClangCL");
        }
        config.static_crt(options.static_crt).define(
            "JPEGXL_RS_STATIC_CRT",
            if options.static_crt { "ON" } else { "OFF" },
        );
    }

    let prefix = config.build();

    let lib_dir = [prefix.join("lib"), prefix.join("lib64")]
        .into_iter()
        .find(|dir| dir.is_dir())
        .unwrap_or_else(|| panic!("Could not find lib or lib64 in {}", prefix.display()));
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    for library in [
        "jxl",
        "jxl_cms",
        "jxl_threads",
        "hwy",
        "brotlidec",
        "brotlienc",
        "brotlicommon",
    ] {
        println!("cargo:rustc-link-lib=static={library}");
    }

    let target_abi = env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default();
    if let Some(runtime) = cpp_runtime(&target_os, &target_vendor, &target_env, &target_abi) {
        println!("cargo:rustc-link-lib={runtime}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msvc_options(vars: &[(&str, &str)]) -> MsvcOptions {
        MsvcOptions::from_cargo(|key| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    #[test]
    fn native_msvc_configuration_uses_the_target_generator_and_static_crt() {
        let options = msvc_options(&[
            ("TARGET", "i686-pc-windows-msvc"),
            ("HOST", "i686-pc-windows-msvc"),
            (
                "CMAKE_GENERATOR_i686-pc-windows-msvc",
                "Visual Studio 18 2026",
            ),
            ("HOST_CMAKE_GENERATOR", "Ninja"),
            ("CARGO_CFG_TARGET_FEATURE", "sse2,crt-static"),
        ]);
        assert!(options.clang_cl_toolset);
        assert!(options.static_crt);
    }

    #[test]
    fn cross_msvc_configuration_uses_the_target_generator_and_dynamic_crt() {
        let vars = [
            ("TARGET", "aarch64-pc-windows-msvc"),
            ("HOST", "x86_64-unknown-linux-gnu"),
            ("TARGET_CMAKE_GENERATOR", "Ninja"),
            ("CMAKE_GENERATOR", "Visual Studio 18 2026"),
        ];
        let options = msvc_options(&vars);
        assert!(!options.clang_cl_toolset);
        assert!(!options.static_crt);
    }

    fn parallel_level(vars: &[(&str, &str)]) -> Option<String> {
        build_parallel_level(|key| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    #[test]
    fn build_parallel_level_keeps_the_one_of_the_environment() {
        assert_eq!(
            parallel_level(&[("CMAKE_BUILD_PARALLEL_LEVEL", "2"), ("NUM_JOBS", "16")]),
            None
        );
    }

    #[test]
    fn build_parallel_level_follows_the_cargo_job_count() {
        assert_eq!(parallel_level(&[("NUM_JOBS", "3")]), Some("3".to_owned()));
    }

    #[test]
    fn build_parallel_level_falls_back_to_the_processor_count() {
        assert_eq!(
            parallel_level(&[]),
            std::thread::available_parallelism()
                .ok()
                .map(|p| p.to_string())
        );
    }

    #[test]
    fn mingw_runtimes_follow_the_target_abi() {
        assert_eq!(cpp_runtime("windows", "pc", "gnu", "llvm"), Some("c++"));
        assert_eq!(cpp_runtime("windows", "pc", "gnu", ""), Some("stdc++"));
        assert_eq!(cpp_runtime("windows", "pc", "msvc", ""), None);
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_source_dir() {
        let mut path = source_dir();
        assert!(path.is_dir());

        path.push("lib/include/jxl/codestream_header.h");
        assert!(path.exists());
    }
}
