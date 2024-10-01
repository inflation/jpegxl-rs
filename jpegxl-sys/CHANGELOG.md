# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.1+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-sys-v0.11.0+libjxl-0.11.0...jpegxl-sys-v0.11.1+libjxl-0.11.0) - 2024-10-01

### Fixed

- Doc build

### Doc

- Remove docsrs-specific reference to deleted "threads" feature

## [0.11.0+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-sys-v0.10.4+libjxl-0.10.3...jpegxl-sys-v0.11.0+libjxl-0.11.0) - 2024-09-27

### Added

- Add gain map utility functions

### Fixed

- Change ffi function types to use `c-unwind` ABI

### Refactor

- Move `libjxl` functions into modules
- Remove threads feature and update dependencies.

### Doc

- Convert `libjxl` doc to rustdoc format with help from @copilot
- Update how docs are generated ([#73](https://github.com/inflation/jpegxl-rs/pull/73))

### Deps

- Bump pkg-config from 0.3.30 to 0.3.31
- Bump pretty_assertions from 1.4.0 to 1.4.1
- Bump image from 0.25.1 to 0.25.2

### CI

- Update release configuration for `jpegxl-rs` and `jpegxl-sys` packages
