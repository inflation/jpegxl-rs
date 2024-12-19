# Changelog

## [Unreleased]

## [0.11.3+libjxl-0.11.1](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.11.2+libjxl-0.11.1...jpegxl-rs-v0.11.3+libjxl-0.11.1)

### ⛰️ Features

- Replace `derive_builder` with `bon` - ([68f0474](https://github.com/inflation/jpegxl-rs/commit/68f047448979409bad51acb0aed9db0475c37e30))

### 🐛 Bug Fixes

- Update `bon` setting to keep from breaking change - ([f743899](https://github.com/inflation/jpegxl-rs/commit/f7438998c69fe585504fdec22e327eec625403c5))
- Ensure to use JxlBool for FFI - ([8f42736](https://github.com/inflation/jpegxl-rs/commit/8f4273692ab02b0f07d8775aa1c9b537a3f27427))

### ⚙️ Miscellaneous Tasks

- Add doc test - ([90a225e](https://github.com/inflation/jpegxl-rs/commit/90a225e154199ef901caf9bb03914a1e3edef136))
- Enable ASan and TSan on `libjxl` - ([ad4542f](https://github.com/inflation/jpegxl-rs/commit/ad4542f9bb47a30a8949a3b5d665a1ad59f71956))
- Enable MSan and TSan (#114) - ([682cc5c](https://github.com/inflation/jpegxl-rs/commit/682cc5c805b6735ac4ab9d48a29f77f902bbe2cb))

## [0.11.2+libjxl-0.11.1](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.11.1+libjxl-0.11.0...jpegxl-rs-v0.11.2+libjxl-0.11.1)

### 📦 Dependencies

- Update `libjxl` to v0.11.1 - ([71f188a](https://github.com/inflation/jpegxl-rs/commit/71f188a331fcbc5c1ec9358ffbcc9e34f6f269c7))

## [0.11.1+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.11.0+libjxl-0.11.0...jpegxl-rs-v0.11.1+libjxl-0.11.0) - 2024-10-01

### 🐛 Bug Fixes

- Fix doc gen

### 📚 Documentation

- Remove docsrs-specific reference to deleted "threads" feature

## [0.11.0+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.10.4+libjxl-0.10.3...jpegxl-rs-v0.11.0+libjxl-0.11.0) - 2024-09-27

### ⛰️ Features

- Update JPEG quality setting in encoder ([#74](https://github.com/inflation/jpegxl-rs/pull/74))

### 🐛 Bug Fixes

- Update release configuration for jpegxl-rs and jpegxl-sys packages
- Change ffi function types to use `c-unwind` ABI
- Change `JxlBoxType` to use system char type

### 🚜 Refactor

- Move `libjxl` functions into modules
- Remove threads feature and update dependencies.
- Don't use `-sys` in `-rs` with default features enabled

### 📚 Documentation

- Convert `libjxl` doc to rustdoc format with help from @copilot
- Remove unnecessary feature attribute in thread pool implementations
- Update how docs are generated ([#73](https://github.com/inflation/jpegxl-rs/pull/73))

### 📦 Dependencies

- Bump thiserror from 1.0.61 to 1.0.64
- Bump pretty_assertions from 1.4.0 to 1.4.1
- Bump derive_builder from 0.20.0 to 0.20.1
- Bump testresult from 0.4.0 to 0.4.1
- Bump image from 0.25.1 to 0.25.2

### ⚙️ Miscellaneous Tasks

- Update release-plz workflow and config
