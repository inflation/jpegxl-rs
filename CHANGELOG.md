# Changelog

<a name="0.8.0"></a>

## 0.8.0 (2023-01-19)

### Added

- ✨ Add utils for checking signature

### Changed

- 🚸 Don't return error if JPEG reconstruction fails
- ⬆️ Upgrade `libjxl` to 0.8.0

<a name="0.7.1"></a>

## 0.7.1 (2022-11-20)

### Added

- ✨ Add resizable threads parallel runner

### Changed

- ⬆️ Upgrade `libjxl` to v0.7

### Miscellaneous

- 📝 Update several docs

<a name="0.7.0-alpha.0"></a>

## 0.7.0-alpha.0 (2022-08-13)

### Changed

- ♻️ Separate source to a separate crate
- ⬆️ Upgrade `libjxl` to 0.7-rc
- ♻️ Move `jpegxl-sys` into workspace
- 🏗️ Use system `libjxl` by default
- ⬆️ Update `image-rs` library

### Miscellaneous

- 📝 Update docs to the latest address

<a name="0.6.1"></a>

## 0.6.1 (2021-11-03)

### Changed

- ⬆️ Upgrade `libjxl` to v0.6.1

### Miscellaneous

- 📝 Add CHANGELOG.md

<a name="0.6.0"></a>

## 0.6.0 (2021-10-13)

### Added

- ✨ Add LUMA only color encoding
- ✨ Automatically figure out pixel types
- ✨ Remove size requirement for JPEG data

### Changed

- ⬆️ Upgrade `libjxl` to v0.6

### Fixed

- 💚 Fix memory leaks
- 🐛 Fix [#9](https://github.com/inflation/jpegxl-rs/issues/9)
- 🐛 Fix potential use-after-free

### Miscellaneous

- 📝 Update docs

<a name="0.3.7"></a>

## 0.3.7 (2021-04-13)

### Changed

- ⬆️ Bump to v0.3.7

### Miscellaneous

- 📝 Update docs

<a name="0.3.5"></a>

## 0.3.5 (2021-03-25)

### Added

- ✨ Check signature first

### Changed

- ⬆️ Upgrade `libjxl` to v0.3.5

<a name="0.3.3"></a>

## 0.3.3 (2021-03-13)

### Added

- ✨ Store JPEG reconstruction metadata
- ✨ Output ICC profile
- ✨ Allow reuse of parallel runner and memory manager

### Changed

- 🎨 Rename feature `without-build` to `system-jpegxl`
- ⬆️ Bump `libjxl` version
- 🎨 Move generic on `JxlDecoder` to decode()

<a name="0.3.2"></a>

## 0.3.2 (2021-02-16)

### Added

- ✨ Add color encoding option

### Changed

- ⬆️ Update `jpeg-xl` to 0.3.1

<a name="0.3.0"></a>

## 0.3.0 (2021-01-30)

### Miscellaneous

- Add API to encode from raw JPEG data lossless
- Update `libjxl` to v0.3

<a name="0.2.3"></a>

## 0.2.3 (2021-01-15)

### Miscellaneous

- Update dependency

<a name="0.2.2"></a>

## 0.2.2 (2021-01-10)

### Miscellaneous

- Add a simple thread pool
- Add encoder building options
- Add Encoder
- Add decoder builder for better API
- Add more `image` crate support
- Add benchmarks

<a name="0.1.4"></a>

## 0.1.4 (2020-08-25)

### Miscellaneous

- Add memory manager

<a name="0.1.2"></a>

## 0.1.2 (2020-08-19)

### Miscellaneous

- Add multithreading runner
- Add GPL-3.0 license and notices

<a name="0.1.0"></a>

## 0.1.0 (2020-08-16)

### Miscellaneous

- 🎉 Initial commit
