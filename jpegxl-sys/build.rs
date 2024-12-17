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

use std::env;

fn main() {
    if cfg!(all(not(feature = "vendored"), not(feature = "docs"))) {
        let version = env!("CARGO_PKG_VERSION")
            .split('+')
            .nth(1)
            .and_then(|s| s.split('-').nth(1))
            .unwrap();

        if let Ok(path) = env::var("DEP_JXL_LIB") {
            println!("cargo:rustc-link-search=native={path}");
            println!("cargo:rustc-link-lib=jxl");
            println!("cargo:rustc-link-lib=jxl_threads");
        } else {
            pkg_config::Config::new()
                .atleast_version(version)
                .probe("libjxl")
                .unwrap_or_else(|_| panic!("Cannot find `libjxl` with version >= {version}"));
            pkg_config::Config::new()
                .atleast_version(version)
                .probe("libjxl_threads")
                .unwrap_or_else(|_| {
                    panic!("Cannot find `libjxl_threads` with version >= {version}")
                });
        }
    } else {
        #[cfg(feature = "vendored")]
        jpegxl_src::build();
    }
}
