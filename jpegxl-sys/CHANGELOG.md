# Changelog

## [Unreleased]

## [0.11.2+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-sys-v0.11.1+libjxl-0.11.0...jpegxl-sys-v0.11.2+libjxl-0.11.0)

### ⚙️ Miscellaneous Tasks

- Updated the following local packages: jpegxl-src - ([0000000](https://github.com/inflation/jpegxl-rs/commit/0000000))

# Changelog

## [Unreleased]

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
