# jpegxl-rs

`jpegxl-rs` is a safe wrapper over `jpeg-xl` library. Check out the original library [here](https://gitlab.com/wg1/jpeg-xl)
and the bindings [here](https://github.com/inflation/jpegxl-sys).

## Building

Install the `jpeg-xl` library system-wide or specify `PKG_CONFIG_PATH` to search for needed paths. Optionally, you can
overwrite the include path and lib path with `DEP_JPEGXL_INCLUDE` and `DEP_JPEGXL_LIB` respectively.

If you want to build the library within cargo, enable `build-jpegxl` features for `jpegxl-sys` in your `Cargo.toml`.

You need to have a working `llvm` environment. Note that this will link to `libc++` by default
(since you already use llvm). You can modify it by setting `DEP_JPEGXL_CXXLIB`.

## Usage

Check out testing in `src/lib.rs` for some examples.

### [`image`](https://crates.io/crates/image) crate integration (WIP)

Enable `with-image` feature. Then you can use `image`'s decoder interface.
