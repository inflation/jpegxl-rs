/*
 * This file is part of jpegxl-rs.
 *
 * jpegxl-rs is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * jpegxl-rs is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
 */

use half::f16;
use image::DynamicImage;
use pretty_assertions::assert_eq;
use testresult::TestResult;

use crate::decode::Data;
use crate::{
    decoder_builder,
    encode::{ColorEncoding, EncoderFrame, EncoderResult, Metadata},
    encoder_builder, Endianness,
};
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
    let threads_runner = ThreadsRunner::default();
    let mut encoder = encoder_builder()
        .parallel_runner(&threads_runner)
        .use_container(true)
        .uses_original_profile(true)
        .build()?;

    let res = encoder.encode_jpeg(super::SAMPLE_JPEG)?;

    let (_, Data::Jpeg(reconstructed)) = decoder_builder().build()?.reconstruct(&res)? else {
        panic!("Failed to reconstruct JPEG");
    };

    assert_eq!(super::SAMPLE_JPEG, reconstructed);

    Ok(())
}

#[test]
fn metadata() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;
    encoder.add_metadata(&Metadata::Exif(super::SAMPLE_EXIF), true)?;
    encoder.add_metadata(&Metadata::Xmp(super::SAMPLE_XMP), true)?;

    let _res: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn builder() -> TestResult {
    use crate::decode::Metadata;

    let sample = get_sample().to_rgba8();
    let threads_runner = ThreadsRunner::default();

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

    Ok(())
}

#[test]
fn resizable() -> TestResult {
    let resizable_runner = ResizableRunner::default();
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder()
        .parallel_runner(&resizable_runner)
        .build()?;

    let _res: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn pixel_type() -> TestResult {
    let mut encoder = encoder_builder().has_alpha(true).build()?;
    let decoder = decoder_builder().build()?;
    let sample = get_sample().to_rgba8();

    // Check different pixel format
    let frame = EncoderFrame::new(sample.as_raw()).num_channels(4);
    let _res: EncoderResult<u16> = encoder.encode_frame(&frame, sample.width(), sample.height())?;
    let _res: EncoderResult<f16> = encoder.encode_frame(&frame, sample.width(), sample.height())?;
    let _res: EncoderResult<f32> = encoder.encode_frame(&frame, sample.width(), sample.height())?;

    encoder.has_alpha = false;
    let sample = get_sample().to_rgb8();
    let _: EncoderResult<u16> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    let res: EncoderResult<f16> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    decoder.decode(&res)?;
    let _: EncoderResult<f32> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    let sample = get_sample().to_rgb32f();
    let _: EncoderResult<f32> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn multi_frames() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().use_container(true).build()?;

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

    encoder.uses_original_profile = true;
    let result: EncoderResult<f32> = encoder
        .multiple(sample.width(), sample.height())?
        .add_jpeg_frame(super::SAMPLE_JPEG)?
        .add_jpeg_frame(super::SAMPLE_JPEG)?
        .encode()?;
    let _res = decoder.reconstruct(&result)?;

    Ok(())
}

#[test]
fn gray() -> TestResult {
    let sample = get_sample().to_luma8();
    let mut encoder = encoder_builder()
        .color_encoding(ColorEncoding::SrgbLuma)
        .build()?;
    let decoder = decoder_builder().build()?;

    let result: EncoderResult<u8> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;
    _ = decoder.decode(&result)?;

    encoder.color_encoding = Some(ColorEncoding::LinearSrgbLuma);
    let result: EncoderResult<u8> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;
    _ = decoder.decode(&result)?;

    encoder.set_frame_option(
        jpegxl_sys::encoder::encode::JxlEncoderFrameSettingId::BrotliEffort,
        1,
    )?;

    Ok(())
}

#[test]
fn initial_buffer() -> TestResult {
    let mut encoder = encoder_builder().init_buffer_size(0).build()?;
    let sample = get_sample().to_rgb8();
    let _: EncoderResult<u16> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    let _: EncoderResult<f16> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    let _: EncoderResult<f32> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    Ok(())
}
