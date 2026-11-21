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
    asan_linker_flags: Option<String>,
}

impl MsvcOptions {
    fn from_cargo(asan: bool, var: impl Fn(&str) -> Option<String>) -> Self {
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
        let asan_linker_flags = asan.then(|| {
            let arch = var("CARGO_CFG_TARGET_ARCH").expect("Cargo target architecture");
            let arch = if arch == "x86" { "i386" } else { &arch };
            format!(
                "clang_rt.asan_dynamic-{arch}.lib clang_rt.asan_dynamic_runtime_thunk-{arch}.lib"
            )
        });
        Self {
            static_crt: target_features
                .split(',')
                .any(|feature| feature == "crt-static"),
            clang_cl_toolset: generator
                .as_deref()
                .map_or(cfg!(windows), |name| name.starts_with("Visual Studio")),
            asan_linker_flags,
        }
    }

    fn runtime_library(&self) -> &'static str {
        if self.static_crt {
            "MultiThreaded"
        } else {
            "MultiThreadedDLL"
        }
    }
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
    config
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("JPEGXL_ENABLE_TOOLS", "OFF")
        .define("JPEGXL_ENABLE_DOXYGEN", "OFF")
        .define("JPEGXL_ENABLE_MANPAGES", "OFF")
        .define("JPEGXL_ENABLE_BENCHMARK", "OFF")
        .define("JPEGXL_ENABLE_EXAMPLES", "OFF")
        .define("JPEGXL_ENABLE_JNI", "OFF")
        .define("JPEGXL_ENABLE_SJPEG", "OFF")
        .define("JPEGXL_ENABLE_OPENEXR", "OFF")
        .define("JPEGXL_BUNDLE_LIBPNG", "OFF");

    if let Ok(p) = std::thread::available_parallelism() {
        config.env("CMAKE_BUILD_PARALLEL_LEVEL", format!("{p}"));
    }

    let asan = env::var_os("CARGO_CFG_ASAN").is_some();
    let tsan = env::var_os("CARGO_CFG_TSAN").is_some();
    let sanitizer = if asan {
        Some(("asan", "address"))
    } else if tsan {
        Some(("tsan", "thread"))
    } else {
        None
    };
    if let Some((name, kind)) = sanitizer {
        let define = kind.to_ascii_uppercase();
        let flags = format!("-g -D{define}_SANITIZER -fsanitize={kind}");
        config
            .env("SANITIZER", name)
            .cflag(&flags)
            .cxxflag(&flags)
            .define("JPEGXL_ENABLE_TCMALLOC", "OFF");
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").expect("Cargo target environment");
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").expect("Cargo target vendor");
    if target_os == "windows" {
        assert!(!tsan, "Thread sanitizer is not supported on Windows");
    }
    if target_env == "msvc" {
        let options = MsvcOptions::from_cargo(asan, |key| env::var(key).ok());
        if let Some(flags) = &options.asan_linker_flags {
            config.define("CMAKE_EXE_LINKER_FLAGS", flags);
        }
        if options.clang_cl_toolset {
            config.generator_toolset("ClangCL").define(
                "CMAKE_VS_GLOBALS",
                "UseMultiToolTask=true;EnforceProcessCountAcrossBuilds=true",
            );
        }
        config
            .static_crt(options.static_crt)
            .define("CMAKE_MSVC_RUNTIME_LIBRARY", options.runtime_library());
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

    fn msvc_options(asan: bool, vars: &[(&str, &str)]) -> MsvcOptions {
        MsvcOptions::from_cargo(asan, |key| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    #[test]
    fn native_msvc_configuration_uses_the_target_generator_and_static_crt() {
        let options = msvc_options(
            true,
            &[
                ("TARGET", "i686-pc-windows-msvc"),
                ("HOST", "i686-pc-windows-msvc"),
                (
                    "CMAKE_GENERATOR_i686-pc-windows-msvc",
                    "Visual Studio 18 2026",
                ),
                ("HOST_CMAKE_GENERATOR", "Ninja"),
                ("CARGO_CFG_TARGET_FEATURE", "sse2,crt-static"),
                ("CARGO_CFG_TARGET_ARCH", "x86"),
            ],
        );
        assert!(options.clang_cl_toolset);
        assert_eq!(options.runtime_library(), "MultiThreaded");
        assert_eq!(
            options.asan_linker_flags.as_deref(),
            Some("clang_rt.asan_dynamic-i386.lib clang_rt.asan_dynamic_runtime_thunk-i386.lib")
        );
    }

    #[test]
    fn cross_msvc_configuration_uses_the_target_architecture_and_generator() {
        let vars = [
            ("TARGET", "aarch64-pc-windows-msvc"),
            ("HOST", "x86_64-unknown-linux-gnu"),
            ("CARGO_CFG_TARGET_ARCH", "aarch64"),
            ("TARGET_CMAKE_GENERATOR", "Ninja"),
            ("CMAKE_GENERATOR", "Visual Studio 18 2026"),
        ];
        let options = msvc_options(true, &vars);
        assert!(!options.clang_cl_toolset);
        assert_eq!(options.runtime_library(), "MultiThreadedDLL");
        assert_eq!(
            options.asan_linker_flags.as_deref(),
            Some(
                "clang_rt.asan_dynamic-aarch64.lib clang_rt.asan_dynamic_runtime_thunk-aarch64.lib"
            )
        );
        assert!(msvc_options(false, &vars).asan_linker_flags.is_none());
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
