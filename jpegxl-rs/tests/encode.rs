use jpegxl_rs::{
    decoder_builder,
    encode::{ColorEncoding, EncoderFrame, EncoderResult, EncoderSpeed},
    encoder_builder, EncodeError, Endianness, ThreadsRunner,
};

use image::{io::Reader as ImageReader, DynamicImage};

fn get_sample() -> DynamicImage {
    ImageReader::open("../samples/sample.png")
        .ok()
        .and_then(|i| i.decode().ok())
        .expect(&format!("Failed to get sample file"))
}

#[test]
fn simple() -> Result<(), EncodeError> {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;

    let result: EncoderResult<u16> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    let decoder = decoder_builder().build().expect("Failed to build decoder");
    let _res = decoder
        .decode(&result)
        .expect("Failed to decode the encoded image");

    Ok(())
}

#[test]
fn jpeg() -> Result<(), EncodeError> {
    let sample = std::fs::read("../samples/sample.jpg").expect("Failed to read sample image file");

    let parallel_runner = ThreadsRunner::default();
    let mut encoder = encoder_builder()
        .use_container(true)
        .parallel_runner(&parallel_runner)
        .build()?;

    let _res = encoder.encode_jpeg(&sample)?;

    Ok(())
}

#[test]
fn builder() -> Result<(), EncodeError> {
    let sample = get_sample().to_rgba8();
    let parallel_runner = ThreadsRunner::default();
    let mut encoder = encoder_builder()
        .has_alpha(true)
        .lossless(false)
        .speed(EncoderSpeed::Lightning)
        .quality(3.0)
        .color_encoding(ColorEncoding::LinearSRgb)
        .decoding_speed(4)
        .init_buffer_size(64)
        .parallel_runner(&parallel_runner)
        .build()?;

    let _res: EncoderResult<u8> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(4),
        sample.width(),
        sample.height(),
    )?;

    // Check encoder reset
    encoder.has_alpha = false;
    let _res: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn multi_frames() -> Result<(), EncodeError> {
    let sample = get_sample().to_rgb8();
    let sample_jpeg =
        std::fs::read("../samples/sample.jpg").expect("Failed to read sample JPEG file");
    let parallel_runner = ThreadsRunner::default();
    let mut encoder = encoder_builder()
        .parallel_runner(&parallel_runner)
        .color_encoding(ColorEncoding::SRgb)
        .build()?;

    let frame = EncoderFrame::new(sample.as_raw())
        .endianness(Endianness::Native)
        .align(0);

    let _res: EncoderResult<u8> = encoder
        .multiple(sample.width(), sample.height())?
        .add_frame(&frame)?
        .add_jpeg_frame(&sample_jpeg)?
        .encode()?;

    Ok(())
}

#[test]
fn gray() -> Result<(), EncodeError> {
    let sample = get_sample().to_luma8();
    let parallel_runner = ThreadsRunner::default();
    let mut encoder = encoder_builder()
        .color_encoding(ColorEncoding::SRgbLuma)
        .parallel_runner(&parallel_runner)
        .build()?;

    let _res: EncoderResult<u8> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;

    // Check encoder reset
    encoder.has_alpha = false;
    let _res: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}
