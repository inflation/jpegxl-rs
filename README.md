# jpegxl-sys

`jpegxl-sys` is a wrapper over `libjxl` library. Check out the original library [here](https://github.com/libjxl/libjxl).

## Building

If you wish to specify a custom library path, set `DEP_JXL_LIB` environment variable.

Building `libjxl` and statically linking can be enabled by using `vendored` feature.
You can provide the source code with all third-party dependencies by `DEP_JXL_PATH`,
or it would fetch the code by `git`.

## Usage

Check out testing units in `src/lib.rs` for some examples.

### Multithread

Because `libjxl_threads` uses `std::thread`, if you build and statically link `libjxl`, you need to
link `libc++` or `libstdc++` standard library as well.
Using dynamic library doesn't need this requirement.

If you don't want the dependency, you can disable the `threads` feature.
