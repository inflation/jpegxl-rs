use half::f16;
use image::DynamicImage;
use testresult::TestResult;

use crate::{
    decoder_builder,
    encode::{ColorEncoding, EncoderFrame, EncoderResult},
    encoder_builder, Endianness,
};
#[cfg(feature = "threads")]
use crate::{encode::EncoderSpeed, ResizableRunner, ThreadsRunner};

fn get_sample() -> DynamicImage {
    image::load_from_memory_with_format(super::SAMPLE_PNG, image::ImageFormat::Png)
        .expect("Failed to get sample file")
}

#[test]
fn simple() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;

    let result: EncoderResult<u16> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    let decoder = decoder_builder().build().expect("Failed to build decoder");
    let _res = decoder.decode(&result)?;

    Ok(())
}

#[test]
fn jpeg() -> TestResult {
    let mut encoder = encoder_builder()
        .use_container(true)
        .uses_original_profile(true)
        .build()?;

    let _res = encoder.encode_jpeg(super::SAMPLE_JPEG)?;

    Ok(())
}

#[test]
#[cfg(feature = "threads")]
fn builder() -> TestResult {
    use crate::decode::Metadata;

    let sample = get_sample().to_rgba8();
    let threads_runner = ThreadsRunner::default();
    let resizable_runner = ResizableRunner::default();

    let mut encoder = encoder_builder()
        .has_alpha(true)
        .lossless(false)
        .speed(EncoderSpeed::Lightning)
        .quality(3.0)
        .color_encoding(ColorEncoding::LinearSrgb)
        .decoding_speed(4)
        .init_buffer_size(64)
        .parallel_runner(&threads_runner)
        .build()?;

    let res: EncoderResult<u8> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(4),
        sample.width(),
        sample.height(),
    )?;

    let decoder = decoder_builder().build().unwrap();
    let (
        Metadata {
            num_color_channels,
            has_alpha_channel,
            ..
        },
        _,
    ) = decoder.decode(&res)?;
    assert_eq!(num_color_channels, 3);
    assert!(has_alpha_channel);

    // Check encoder reset
    encoder.has_alpha = false;
    let sample = get_sample().to_rgb8();

    let _res: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    // Check different pixel format
    let _res: EncoderResult<u16> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    let _res: EncoderResult<f16> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    let _res: EncoderResult<f32> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    // Check resizable runner
    encoder.parallel_runner = Some(&resizable_runner);
    let _res: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn multi_frames() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder()
        .color_encoding(ColorEncoding::Srgb)
        .build()?;

    let frame = EncoderFrame::new(sample.as_raw())
        .endianness(Endianness::Native)
        .align(0);

    let result: EncoderResult<f32> = encoder
        .multiple(sample.width(), sample.height())?
        .add_frame(&frame)?
        .add_frame(&frame)?
        .encode()?;

    let decoder = decoder_builder().build()?;
    let _res = decoder.decode(&result)?;

    Ok(())
}

#[test]
fn gray() -> TestResult {
    let sample = get_sample().to_luma8();
    let mut encoder = encoder_builder()
        .color_encoding(ColorEncoding::SrgbLuma)
        .build()?;

    let result: EncoderResult<f16> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;

    let decoder = decoder_builder().build()?;
    let _res = decoder.decode(&result)?;

    encoder.color_encoding = ColorEncoding::LinearSrgbLuma;
    let result: EncoderResult<f16> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;

    let _res = decoder.decode(&result)?;

    Ok(())
}
