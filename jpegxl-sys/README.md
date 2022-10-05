# jpegxl-sys

`jpegxl-sys` is a wrapper over `libjxl` library. Check out the original library [here](https://github.com/libjxl/libjxl).

## Building

If you wish to specify a custom library path, set the `DEP_JXL_LIB` environment variable.

Building `libjxl` and statically linking can be enabled by using the `vendored` feature.

## Usage

Check out testing units in `src/lib.rs` for some examples.

### Multi-thread

Because `libjxl_threads` uses `std::thread`, if you build and statically link `libjxl`, you need to
dynamically link to `libc++` or `libstdc++`.

Using dynamic library doesn't need this requirement.
