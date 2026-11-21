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

fn configure_msvc(config: &mut cmake::Config, asan: bool) {
    let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let static_crt = target_features
        .split(',')
        .any(|feature| feature == "crt-static");
    let runtime = if static_crt {
        "MultiThreaded"
    } else {
        "MultiThreadedDLL"
    };
    if asan {
        let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo target architecture");
        let arch = if arch == "x86" { "i386" } else { &arch };
        let linker_flags = format!(
            "clang_rt.asan_dynamic-{arch}.lib clang_rt.asan_dynamic_runtime_thunk-{arch}.lib"
        );
        config.define("CMAKE_EXE_LINKER_FLAGS", linker_flags);
    }
    let target = env::var("TARGET").expect("Cargo target triple");
    let build_kind = if env::var("HOST").as_ref() == Ok(&target) {
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
    .find_map(|key| env::var(key).ok());
    if generator
        .as_deref()
        .map_or(cfg!(windows), |name| name.starts_with("Visual Studio"))
    {
        config.generator_toolset("ClangCL").define(
            "CMAKE_VS_GLOBALS",
            "UseMultiToolTask=true;EnforceProcessCountAcrossBuilds=true",
        );
    }
    config
        .static_crt(static_crt)
        .define("CMAKE_MSVC_RUNTIME_LIBRARY", runtime);
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
        configure_msvc(&mut config, asan);
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
