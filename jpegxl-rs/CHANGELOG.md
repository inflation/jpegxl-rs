# Changelog

## [Unreleased]

## [0.13.1+libjxl-0.11.2]

### Dep

- Update `libjxl` to v0.11.2 - ([d45159f](https://github.com/inflation/jpegxl-rs/commit/d45159f0552b393289aa964b617fe7d6747a32ee))


## [0.13.0+libjxl-0.11.1](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.12.0+libjxl-0.11.1...jpegxl-rs-v0.13.0+libjxl-0.11.1)

### ⛰️ Features

- Allow custom color encoding - ([2db5bfe](https://github.com/inflation/jpegxl-rs/commit/2db5bfe9e817b1d066aad966770bb1c80633d226))

### Build

- *(deps)* Bump thiserror from 2.0.17 to 2.0.18 - ([d2895e1](https://github.com/inflation/jpegxl-rs/commit/d2895e1eeb927bd69d1964017fad982d398fe1a2))
- *(deps)* Bump bon from 3.8.1 to 3.8.2 - ([5482433](https://github.com/inflation/jpegxl-rs/commit/5482433da202dae625e3d0965d2a5d829a355ee5))


## [0.12.0+libjxl-0.11.1](https://github.com/inflation/jpegxl-rs/compare/jpegxl-rs-v0.11.2+libjxl-0.11.1...jpegxl-rs-v0.12.0+libjxl-0.11.1)

### ⛰️ Features

- Implement Send for JxlDecoder and JxlEncoder - ([f855779](https://github.com/inflation/jpegxl-rs/commit/f8557795ff89d4fe50f4140c201ca29d7bb85fd0))
- Add #[non_exhaustive] to error enums - ([d5e538c](https://github.com/inflation/jpegxl-rs/commit/d5e538c71e54aab87b0c6b9a9df7bf6057445e08))
- Replace `derive_builder` with `bon` - ([68f0474](https://github.com/inflation/jpegxl-rs/commit/68f047448979409bad51acb0aed9db0475c37e30))

### 🐛 Bug Fixes

- Replace todo!() panics with NotImplemented errors - ([f53222a](https://github.com/inflation/jpegxl-rs/commit/f53222a4e2b02ca10018cd748a851213f0b1a845))
- Update `bon` setting to keep from breaking change - ([f743899](https://github.com/inflation/jpegxl-rs/commit/f7438998c69fe585504fdec22e327eec625403c5))
- Ensure to use JxlBool for FFI - ([8f42736](https://github.com/inflation/jpegxl-rs/commit/8f4273692ab02b0f07d8775aa1c9b537a3f27427))

### 🚜 Refactor

- Bump MSRV to 1.85 (Debian Trixie) - ([b6fdbad](https://github.com/inflation/jpegxl-rs/commit/b6fdbad2dbeb2c25d718aa865a2980d4423bbc15))

### ⚙️ Miscellaneous Tasks

- Add doc test - ([90a225e](https://github.com/inflation/jpegxl-rs/commit/90a225e154199ef901caf9bb03914a1e3edef136))
- Enable ASan and TSan on `libjxl` - ([ad4542f](https://github.com/inflation/jpegxl-rs/commit/ad4542f9bb47a30a8949a3b5d665a1ad59f71956))
- Enable MSan and TSan ([#114](https://github.com/inflation/jpegxl-rs/pull/114)) - ([682cc5c](https://github.com/inflation/jpegxl-rs/commit/682cc5c805b6735ac4ab9d48a29f77f902bbe2cb))

### Build

- *(deps)* Bump bon from 3.8.0 to 3.8.1 - ([60f71f1](https://github.com/inflation/jpegxl-rs/commit/60f71f1506a57b584186163530ca88c470bd7163))
- *(deps)* Bump bon from 3.7.2 to 3.8.0 - ([10f441c](https://github.com/inflation/jpegxl-rs/commit/10f441c952b6fd4e7821caf0a99baeb8f2fa6958))
- *(deps)* Bump thiserror from 2.0.16 to 2.0.17 - ([c8b1b49](https://github.com/inflation/jpegxl-rs/commit/c8b1b490c8258853b52571cfc7e49cb34ae38276))
- *(deps)* Bump bon from 3.7.1 to 3.7.2 - ([28ef7d2](https://github.com/inflation/jpegxl-rs/commit/28ef7d27229c5899473e4d79d5e58f54016d79be))
- *(deps)* Bump bon from 3.7.0 to 3.7.1 - ([3aa46a4](https://github.com/inflation/jpegxl-rs/commit/3aa46a410351a8430581f2e6bfc4b214f06f39ae))
- *(deps)* Bump thiserror from 2.0.15 to 2.0.16 - ([f5819a1](https://github.com/inflation/jpegxl-rs/commit/f5819a1aaaf49b99bb49bb743dfb30dc9a6ed9dd))
- *(deps)* Bump thiserror from 2.0.12 to 2.0.15 - ([8aeac92](https://github.com/inflation/jpegxl-rs/commit/8aeac92f4aea728585fc58fefd9cc8cb9b5b80be))
- *(deps)* Bump bon from 3.6.5 to 3.7.0 - ([35c6d7e](https://github.com/inflation/jpegxl-rs/commit/35c6d7e529185649988d81935aee4e98c35f2729))
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
- Bump MSRV to 1.81 - ([f8f16e9](https://github.com/inflation/jpegxl-rs/commit/f8f16e9f952e2577703d4ae64fb571443a00a13e))


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
