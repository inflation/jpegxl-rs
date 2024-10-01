# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.1+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.11.0+libjxl-0.11.0...jpegxl-rs-v0.11.1+libjxl-0.11.0) - 2024-10-01

### Fixed

- Fix doc gen

### Doc

- Remove docsrs-specific reference to deleted "threads" feature

## [0.11.0+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.10.4+libjxl-0.10.3...jpegxl-rs-v0.11.0+libjxl-0.11.0) - 2024-09-27

### Added

- Update JPEG quality setting in encoder ([#74](https://github.com/inflation/jpegxl-rs/pull/74))

### Fixed

- Update release configuration for jpegxl-rs and jpegxl-sys packages
- Change ffi function types to use `c-unwind` ABI
- Change `JxlBoxType` to use system char type

### Refactor

- Move `libjxl` functions into modules
- Remove threads feature and update dependencies.
- Don't use `-sys` in `-rs` with default features enabled

### Doc

- Convert `libjxl` doc to rustdoc format with help from @copilot
- Remove unnecessary feature attribute in thread pool implementations
- Update how docs are generated ([#73](https://github.com/inflation/jpegxl-rs/pull/73))

### Deps

- Bump thiserror from 1.0.61 to 1.0.64
- Bump pretty_assertions from 1.4.0 to 1.4.1
- Bump derive_builder from 0.20.0 to 0.20.1
- Bump testresult from 0.4.0 to 0.4.1
- Bump image from 0.25.1 to 0.25.2

### CI

- Update release-plz workflow and config
