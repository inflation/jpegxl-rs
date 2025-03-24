# Changelog

## [Unreleased]

## [0.12.0+libjxl-0.11.1](https://github.com/inflation/jpegxl-rs/compare/jpegxl-sys-v0.11.2+libjxl-0.11.1...jpegxl-sys-v0.12.0+libjxl-0.11.1)

### 🐛 Bug Fixes

- Ensure to use JxlBool for FFI - ([8f42736](https://github.com/inflation/jpegxl-rs/commit/8f4273692ab02b0f07d8775aa1c9b537a3f27427))

### ⚙️ Miscellaneous Tasks

- Enable ASan and TSan on `libjxl` - ([ad4542f](https://github.com/inflation/jpegxl-rs/commit/ad4542f9bb47a30a8949a3b5d665a1ad59f71956))

## [0.11.2+libjxl-0.11.1](https://github.com/inflation/jpegxl-rs/compare/jpegxl-sys-v0.11.1+libjxl-0.11.0...jpegxl-sys-v0.11.2+libjxl-0.11.1)

### 🐛 Bug Fixes

- Update extern "C" to extern "C-unwind" in multiple files - ([793c23a](https://github.com/inflation/jpegxl-rs/commit/793c23a04b5f167ec46875cdc85e5b6eb64b260a))

### 📦 Dependencies

- Update `libjxl` to v0.11.1 - ([71f188a](https://github.com/inflation/jpegxl-rs/commit/71f188a331fcbc5c1ec9358ffbcc9e34f6f269c7))

## [0.11.1+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-sys-v0.11.0+libjxl-0.11.0...jpegxl-sys-v0.11.1+libjxl-0.11.0) - 2024-10-01

### 🐛 Bug Fixes

- Fix doc gen

### 📚 Documentation

- Remove docsrs-specific reference to deleted "threads" feature

## [0.11.0+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-sys-v0.10.4+libjxl-0.10.3...jpegxl-sys-v0.11.0+libjxl-0.11.0) - 2024-09-27

### ⛰️ Features

- Add gain map utility functions

### 🐛 Bug Fixes

- Change ffi function types to use `c-unwind` ABI

### 🚜 Refactor

- Move `libjxl` functions into modules
- Remove threads feature and update dependencies.

### 📚 Documentation

- Convert `libjxl` doc to rustdoc format with help from @copilot
- Update how docs are generated ([#73](https://github.com/inflation/jpegxl-rs/pull/73))

### 📦 Dependencies

- Bump pkg-config from 0.3.30 to 0.3.31
- Bump pretty_assertions from 1.4.0 to 1.4.1
- Bump image from 0.25.1 to 0.25.2

### ⚙️ Miscellaneous Tasks

- Update release configuration for `jpegxl-rs` and `jpegxl-sys` packages
