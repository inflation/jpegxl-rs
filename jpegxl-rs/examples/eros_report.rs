//! Prints what an eros error looks like to a user of this crate
use jpegxl_rs::{
    decoder_builder,
    encode::{EncoderFailure, JxlEncoder},
    encoder_builder,
    eros::{self, Context},
};

/// Recompress a JPEG losslessly, or encode its pixels if it cannot be recompressed
fn jpeg_to_jxl(encoder: &mut JxlEncoder, jpeg: &[u8]) -> eros::Result<Vec<u8>> {
    let error = match encoder.encode_jpeg(jpeg) {
        Ok(data) => return Ok(data),
        Err(error) => error,
    };
    match error.narrow::<EncoderFailure, _>() {
        Ok(EncoderFailure::Jbrd | EncoderFailure::NotSupported) => {
            println!("cannot recompress, encoding the pixels instead");
            let image = image::load_from_memory(jpeg)?.to_rgb8();
            Ok(encoder.encode(image.as_raw(), image.width(), image.height())?)
        }
        Ok(failure) => Err(failure.into()),
        Err(rest) => Err(rest.into()),
    }
}

fn main() {
    let decoder = decoder_builder().build().unwrap();
    let sample = include_bytes!("../../samples/sample.jxl");

    let error = decoder
        .decode(&sample[..100])
        .context("load thumbnail.jxl")
        .unwrap_err();
    println!("--- Display ---\n{error}\n--- Debug ---\n{error:?}");

    let mut encoder = encoder_builder().build().unwrap();
    let cmyk = include_bytes!("../../samples/sample_cmyk.jpg");
    let data = jpeg_to_jxl(&mut encoder, cmyk).unwrap();
    println!("--- JPEG fallback ---\n{} bytes", data.len());
}
