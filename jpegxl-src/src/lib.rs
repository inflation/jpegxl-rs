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

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn build() {
    let source = source_dir();

    let mut config = cmake::Config::new(source);
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
        .define("JPEGXL_ENABLE_JPEGLI", "OFF")
        .define("JPEGXL_BUNDLE_LIBPNG", "OFF");

    if let Ok(p) = std::thread::available_parallelism() {
        config.env("CMAKE_BUILD_PARALLEL_LEVEL", format!("{p}"));
    }

    #[cfg(target_os = "windows")]
    {
        config
            .generator_toolset("ClangCL")
            .define(
                "CMAKE_VS_GLOBALS",
                "UseMultiToolTask=true;EnforceProcessCountAcrossBuilds=true",
            ) // Enable parallel builds
            .define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded")
            .define("CMAKE_EXE_LINKER_FLAGS", "MSVCRTD.lib")
            .cflag("-Zl");
    }

    let mut prefix = config.build();
    prefix.push("lib");
    println!("cargo:rustc-link-search=native={}", prefix.display());
    prefix.pop();
    prefix.push("lib64");
    println!("cargo:rustc-link-search=native={}", prefix.display());

    println!("cargo:rustc-link-lib=static=jxl");
    println!("cargo:rustc-link-lib=static=jxl_cms");

    #[cfg(feature = "threads")]
    println!("cargo:rustc-link-lib=static=jxl_threads");

    println!("cargo:rustc-link-lib=static=hwy");
    println!("cargo:rustc-link-lib=static=brotlicommon");
    println!("cargo:rustc-link-lib=static=brotlidec");
    println!("cargo:rustc-link-lib=static=brotlienc");
    #[cfg(any(target_vendor = "apple", target_os = "freebsd"))]
    {
        println!("cargo:rustc-link-lib=c++");
    }
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=stdc++");
    }
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
