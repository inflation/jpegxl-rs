//! Prints what an eros error looks like to a user of this crate
use jpegxl_rs::{decoder_builder, encoder_builder, eros::Context, errors::EncoderStatus};

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
    match error.narrow::<EncoderStatus, _>() {
        Ok(EncoderStatus(code)) => println!("libjxl rejected it with {code:?}"),
        Err(rest) => println!("something else: {rest}"),
    }

    println!(
        "--- sizes ---\nResult<(), old DecodeError-like enum>: {}\neros::Result<(), DecodeError>: {}",
        std::mem::size_of::<Result<(), (u64, &'static str)>>(),
        std::mem::size_of::<jpegxl_rs::eros::Result<(), jpegxl_rs::DecodeError>>(),
    );
}
