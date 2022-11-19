use half::f16;
use image::DynamicImage;

use crate::{
    decoder_builder,
    encode::{ColorEncoding, EncoderFrame, EncoderResult},
    encoder_builder, EncodeError, Endianness,
};
#[cfg(feature = "threads")]
use crate::{encode::EncoderSpeed, ResizableRunner, ThreadsRunner};

use super::SAMPLE_JPEG;

fn get_sample() -> DynamicImage {
    image::load_from_memory_with_format(super::SAMPLE_PNG, image::ImageFormat::Png)
        .expect("Failed to get sample file")
}

#[test]
fn simple() {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build().unwrap();

    let result: EncoderResult<u16> = encoder
        .encode(sample.as_raw(), sample.width(), sample.height())
        .unwrap();

    let decoder = decoder_builder().build().expect("Failed to build decoder");
    let _res = decoder
        .decode(&result)
        .expect("Failed to decode the encoded image");
}

#[test]
fn jpeg() -> Result<(), EncodeError> {
    let mut encoder = encoder_builder().use_container(true).build()?;

    let _res = encoder.encode_jpeg(super::SAMPLE_JPEG)?;

    Ok(())
}

#[test]
#[cfg(feature = "threads")]
fn builder() {
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
        .build()
        .unwrap();

    let res: EncoderResult<u8> = encoder
        .encode_frame(
            &EncoderFrame::new(sample.as_raw()).num_channels(4),
            sample.width(),
            sample.height(),
        )
        .unwrap();

    let decoder = decoder_builder().build().unwrap();
    let dec_res = decoder.decode(&res).unwrap();
    assert!(dec_res.num_channels == 4);

    // Check encoder reset
    encoder.has_alpha = false;
    let sample = get_sample().to_rgb8();

    let _res: EncoderResult<u8> = encoder
        .encode(sample.as_raw(), sample.width(), sample.height())
        .unwrap();

    // Check different pixel format
    let _res: EncoderResult<u16> = encoder
        .encode(sample.as_raw(), sample.width(), sample.height())
        .unwrap();

    let _res: EncoderResult<f16> = encoder
        .encode(sample.as_raw(), sample.width(), sample.height())
        .unwrap();

    let _res: EncoderResult<f32> = encoder
        .encode(sample.as_raw(), sample.width(), sample.height())
        .unwrap();

    // Check resizable runner
    encoder.parallel_runner = Some(&resizable_runner);
    let _res: EncoderResult<u8> = encoder
        .encode(sample.as_raw(), sample.width(), sample.height())
        .unwrap();
}

#[test]
fn multi_frames() {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder()
        .color_encoding(ColorEncoding::Srgb)
        .build()
        .unwrap();

    let frame = EncoderFrame::new(sample.as_raw())
        .endianness(Endianness::Native)
        .align(0);

    let result: EncoderResult<f32> = encoder
        .multiple(sample.width(), sample.height())
        .unwrap()
        .add_frame(&frame)
        .unwrap()
        .add_jpeg_frame(SAMPLE_JPEG)
        .unwrap()
        .encode()
        .unwrap();

    let decoder = decoder_builder().build().unwrap();
    let _res = decoder.decode(&result).unwrap();
}

#[test]
fn gray() {
    let sample = get_sample().to_luma8();
    let mut encoder = encoder_builder()
        .color_encoding(ColorEncoding::SrgbLuma)
        .build()
        .unwrap();

    let result: EncoderResult<f16> = encoder
        .encode_frame(
            &EncoderFrame::new(sample.as_raw()).num_channels(1),
            sample.width(),
            sample.height(),
        )
        .unwrap();

    let decoder = decoder_builder().build().unwrap();
    let _res = decoder.decode(&result).unwrap();

    encoder.color_encoding = ColorEncoding::LinearSrgbLuma;
    let result: EncoderResult<f16> = encoder
        .encode_frame(
            &EncoderFrame::new(sample.as_raw()).num_channels(1),
            sample.width(),
            sample.height(),
        )
        .unwrap();

    let _res = decoder.decode(&result).unwrap();
}
