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

//! Build script for jpegxl-sys.

fn main() {
    #[cfg(feature = "vendored")]
    jpegxl_src::build();

    if cfg!(not(any(feature = "vendored", feature = "docs"))) {
        link_system_libraries();
    }
}

fn link_system_libraries() {
    let (_, version) = env!("CARGO_PKG_VERSION")
        .split_once("+libjxl-")
        .expect("package version includes the libjxl version");
    println!("cargo:rerun-if-env-changed=DEP_JXL_LIB");
    if let Ok(path) = std::env::var("DEP_JXL_LIB") {
        println!("cargo:rustc-link-search=native={path}");
        println!("cargo:rustc-link-lib=jxl");
        println!("cargo:rustc-link-lib=jxl_threads");
        println!("cargo:rustc-link-lib=jxl_cms");
    } else {
        for library in ["libjxl", "libjxl_threads", "libjxl_cms"] {
            pkg_config::Config::new()
                .atleast_version(version)
                .probe(library)
                .unwrap_or_else(|_| panic!("Cannot find `{library}` with version >= {version}"));
        }
    }
}
