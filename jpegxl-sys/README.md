# `jpegxl-sys`

`jpegxl-sys` provides a wrapper for the `libjxl` library.
For more details, refer to the [original library](https://github.com/libjxl/libjxl).

## Building

If you wish to specify a custom library path, set the `DEP_JXL_LIB` environment variable.

To build `libjxl` and statically link it, use the `vendored` feature.

## Usage

Check out testing units in `src/lib.rs` for some examples.

### Multi-thread

Because `libjxl_threads` uses `std::thread`, if you build and statically link `libjxl`, you need to
dynamically link to `libc++` or `libstdc++`.

Using the dynamic library doesn't need this step.
