# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.1+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.11.0+libjxl-0.11.0...jpegxl-rs-v0.11.1+libjxl-0.11.0) - 2024-09-30

### Other

- One more!
- remove docsrs-specific reference to deleted "threads" feature

## [0.11.0+libjxl-0.11.0](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.10.4+libjxl-0.10.3...jpegxl-rs-v0.11.0+libjxl-0.11.0) - 2024-09-27

### Added

- Update JPEG quality setting in encoder ([#74](https://github.com/inflation/jpegxl-rs/pull/74))

### Fixed

- Update release configuration for jpegxl-rs and jpegxl-sys packages
- Change ffi function types to use `c-unwind` ABI

### Other

- Convert `libjxl` doc to rustdoc format with help from @copilot
- Move `libjxl` functions into modules
- Bump thiserror from 1.0.63 to 1.0.64
- Bump pretty_assertions from 1.4.0 to 1.4.1
- Remove unnecessary feature attribute in thread pool implementations
- Remove threads feature and update dependencies.
- Don't use -sys in -rs with default features enabled
- Bump derive_builder from 0.20.0 to 0.20.1
- Change JxlBoxType to use system char type
- Update how docs are generated ([#73](https://github.com/inflation/jpegxl-rs/pull/73))
- ⬆️ (deps): Bump thiserror from 1.0.62 to 1.0.63 ([#66](https://github.com/inflation/jpegxl-rs/pull/66))
- :arrow_up: (deps): Bump testresult from 0.4.0 to 0.4.1
- :arrow_up: (deps): Bump image from 0.25.1 to 0.25.2
- :arrow_up: (deps): Bump thiserror from 1.0.61 to 1.0.62
- Update release-plz workflow and config
