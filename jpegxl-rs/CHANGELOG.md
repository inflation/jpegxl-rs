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
- Enable MSan and TSan ([#114](https://github.com/inflation/jpegxl-rs/pull/114)) - ([682cc5c](https://github.com/inflation/jpegxl-rs/commit/682cc5c805b6735ac4ab9d48a29f77f902bbe2cb))

### Build

- *(deps)* Bump criterion from 0.5.1 to 0.7.0 - ([d6c9bb5](https://github.com/inflation/jpegxl-rs/commit/d6c9bb576409ca6f4f37a53c22e7fe1ee5010fbd))
- *(deps)* Bump bon from 3.6.4 to 3.6.5 - ([7306129](https://github.com/inflation/jpegxl-rs/commit/7306129425fe3d1111f1621ccca2520d22b7217c))
- *(deps)* Bump bon from 3.6.3 to 3.6.4 - ([a9e41b1](https://github.com/inflation/jpegxl-rs/commit/a9e41b157b3d9595a6c56e1f6f884d00d95661b2))
- *(deps)* Bump lcms2 from 6.1.0 to 6.1.1 - ([70f6bb5](https://github.com/inflation/jpegxl-rs/commit/70f6bb5a9c2a3ec987246cd59d5016931b589a6f))
- *(deps)* Bump bon from 3.6.0 to 3.6.3 - ([b07ec84](https://github.com/inflation/jpegxl-rs/commit/b07ec84264d7c88d02c1b636d2751e8c61f84384))
- *(deps)* Bump bon from 3.5.1 to 3.6.0 - ([5276a12](https://github.com/inflation/jpegxl-rs/commit/5276a1247e678e5de29ea2835ecbf6917124a5bf))
- *(deps)* Bump image from 0.25.5 to 0.25.6 - ([2f3732a](https://github.com/inflation/jpegxl-rs/commit/2f3732af4d68a08662301bbc249a5cfa1d1ab2fa))
- *(deps)* Bump bon from 3.5.0 to 3.5.1 - ([92ff90a](https://github.com/inflation/jpegxl-rs/commit/92ff90accdd9bfcb49ad75ff42ca984b239fc16e))
- *(deps)* Bump bon from 3.4.0 to 3.5.0 - ([797af62](https://github.com/inflation/jpegxl-rs/commit/797af62d87d2ee2168091a4427c9f04cba987b24))
- *(deps)* Bump bon from 3.3.2 to 3.4.0 - ([224b344](https://github.com/inflation/jpegxl-rs/commit/224b344c9e3ec077be0718493bc286be193c2a86))
- *(deps)* Bump thiserror from 2.0.11 to 2.0.12 - ([60a0b11](https://github.com/inflation/jpegxl-rs/commit/60a0b116dd588894a05cae1cf2cf6b832e121283))
- *(deps)* Bump thiserror from 2.0.9 to 2.0.11 - ([bccd1a3](https://github.com/inflation/jpegxl-rs/commit/bccd1a31de6d9fa436ac49164c4863909d741faf))
- *(deps)* Bump bon from 3.3.1 to 3.3.2 - ([cca78c7](https://github.com/inflation/jpegxl-rs/commit/cca78c78e192b36ba183782e83015fffed01be9f))
- *(deps)* Bump thiserror from 2.0.7 to 2.0.9 - ([f83c4af](https://github.com/inflation/jpegxl-rs/commit/f83c4afe3af01233da3285c8ed7091bf74fd4a6e))
- *(deps)* Bump bon from 3.3.0 to 3.3.1 - ([cad7eb7](https://github.com/inflation/jpegxl-rs/commit/cad7eb7cb1357d34246ff2cfb4c4b4c84ed03554))
- *(deps)* Bump thiserror from 2.0.6 to 2.0.7 - ([fcb1707](https://github.com/inflation/jpegxl-rs/commit/fcb1707bd8606430b1a4407ea9012f0b207e5a82))
- *(deps)* Bump bon from 3.2.0 to 3.3.0 - ([f8cc43c](https://github.com/inflation/jpegxl-rs/commit/f8cc43c55b5ed9483ae737f4a82f183cebf0a094))
- *(deps)* Bump thiserror from 2.0.3 to 2.0.6 - ([35e821b](https://github.com/inflation/jpegxl-rs/commit/35e821bd0ea963a2a08e5906203ed1d6e83e7433))
- Bump MSRV to 1.81 - ([cc7a416](https://github.com/inflation/jpegxl-rs/commit/cc7a416c50a0ee514208f58f144bbf54603f7d90))

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
