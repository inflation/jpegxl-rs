//! Prints what an eros error looks like to a user of this crate
use jpegxl_rs::{
    decoder_builder, encoder_builder,
    eros::Context,
    errors::{
        GenericError, IncompleteInput, InternalError, InvalidInput, InvalidState, UnexpectedStatus,
        UnsupportedBitWidth,
    },
};

fn main() {
    let decoder = decoder_builder().build().unwrap();
    let sample = include_bytes!("../../samples/sample.jxl");

    let error = decoder
        .decode(&sample[..100])
        .context("load thumbnail.jxl")
        .unwrap_err();
    println!("--- Display ---\n{error}\n--- Debug ---\n{error:?}");

    let mut encoder = encoder_builder().build().unwrap();
    let error = encoder.encode::<u8>(&[], 0, 0).unwrap_err();
    println!("--- Display ---\n{error}");
    // Typed recovery: only the encoder status is interesting here
    match error.narrow::<jpegxl_rs::errors::ApiUsage, _>() {
        Ok(e) => println!("libjxl rejected it: {e:?}"),
        Err(rest) => println!("something else: {rest}"),
    }

    println!(
        "--- sizes ---\nResult<(), old DecodeError-like enum>: {}\neros::Result<(), (7 decode failures)>: {}",
        std::mem::size_of::<Result<(), (u64, &'static str)>>(),
        std::mem::size_of::<
            jpegxl_rs::eros::Result<
                (),
                (
                    InvalidInput,
                    IncompleteInput,
                    GenericError,
                    UnexpectedStatus,
                    UnsupportedBitWidth,
                    InvalidState,
                    InternalError,
                ),
            >,
        >(),
    );
}
