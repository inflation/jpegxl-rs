# `jpegxl-sys`

`jpegxl-sys` provides a wrapper for the `libjxl` library.
For more details, refer to the [original library](https://github.com/libjxl/libjxl).

## Building

If you wish to specify a custom library path, set the `DEP_JXL_LIB` environment variable.

To build and statically link `libjxl`, enable `vendored`. When supplying your own
static archives through `DEP_JXL_LIB`, also link their dependencies, including the
C++ runtime used by the native toolchain.

The vendored archive includes the gain-map and compressed ICC functions. For a custom
system installation that provides these upstream extras symbols, enable `system-extras`
and link any separate extras archive supplied by that installation.

## Usage

The tests in `src/lib.rs` demonstrate encoding, decoding, and thread-pool setup.
