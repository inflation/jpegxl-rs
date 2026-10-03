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
use jpegxl_sys::{
    color::color_encoding::{
        JxlColorEncoding, JxlColorSpace, JxlPrimaries, JxlRenderingIntent, JxlTransferFunction,
        JxlWhitePoint,
    },
    encoder::encode::JxlEncoderFrameSettingId,
};
use pretty_assertions::assert_eq;
use testresult::TestResult;

use crate::decode::{BasicInfo, Data, Event, Events};
use crate::DecodeError;
use crate::{
    decoder_builder,
    encode::{ColorEncoding, EncoderFrame, FrameSettings, ImageInfo, Metadata},
    encoder_builder, EncodeError, Endianness,
};
use crate::{encode::EncoderSpeed, ResizableRunner, ThreadsRunner};

fn get_sample() -> DynamicImage {
    image::load_from_memory_with_format(super::SAMPLE_PNG, image::ImageFormat::Png)
        .expect("Failed to get sample file")
}

fn basic_info(data: &[u8]) -> Result<BasicInfo, DecodeError> {
    let mut decoder = decoder_builder().build()?;
    let mut session = decoder.session(Events::empty())?;
    let mut input = data;
    loop {
        if let Event::BasicInfo(info) = session.process(&mut input)? {
            return Ok(info);
        }
    }
}

fn box_types(data: &[u8]) -> Result<Vec<[u8; 4]>, DecodeError> {
    let mut decoder = decoder_builder().build()?;
    let mut session = decoder.session(Events::BOX)?;
    session.close_input();
    let mut input = data;
    let mut types = vec![];
    loop {
        match session.process(&mut input)? {
            Event::Box(header) => types.push(header.box_type),
            Event::Success => return Ok(types),
            _ => {}
        }
    }
}

#[test]
fn simple() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;

    let result = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

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

// Recompressing a JPEG with an unsupported feature (here: 4-component CMYK)
// must fail with a specific error, not succeed or crash. In libjxl v0.12.0
// the component-count check fails while serializing the JPEG reconstruction
// data (`Jbrd`); upstream is moving such failures to `NotSupported`
#[test]
fn jpeg_unsupported_features() -> TestResult {
    let mut encoder = encoder_builder().build()?;

    assert!(matches!(
        encoder.encode_jpeg(super::SAMPLE_JPEG_CMYK),
        Err(EncodeError::Jbrd | EncodeError::NotSupported)
    ));

    Ok(())
}

#[test]
fn metadata() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;
    encoder.add_metadata(&Metadata::Exif(super::SAMPLE_EXIF), true);
    encoder.add_metadata(&Metadata::Xmp(super::SAMPLE_XMP), true);
    encoder.add_metadata(&Metadata::Jumb(b"jumb"), false);
    encoder.add_metadata(&Metadata::Custom(*b"abcd", b"custom"), false);

    let _res = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

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

    let res = encoder.encode_frame(
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

    let _res = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn pixel_type() -> TestResult {
    let mut encoder = encoder_builder().has_alpha(true).build()?;
    let decoder = decoder_builder().build()?;
    let sample = get_sample();
    let (w, h) = (sample.width(), sample.height());

    let rgba16 = sample.to_rgba16();
    let res = encoder.encode_frame(&EncoderFrame::new(rgba16.as_raw()).num_channels(4), w, h)?;
    assert_eq!(basic_info(&res)?.bits_per_sample, 16);
    let rgba32f = sample.to_rgba32f();
    let res = encoder.encode_frame(&EncoderFrame::new(rgba32f.as_raw()).num_channels(4), w, h)?;
    assert_eq!(basic_info(&res)?.exponent_bits_per_sample, 8);
    decoder.decode(&res)?;

    encoder.has_alpha = false;
    let half: Vec<f16> = sample
        .to_rgb32f()
        .as_raw()
        .iter()
        .map(|&v| f16::from_f32(v))
        .collect();
    let res = encoder.encode(&half, w, h)?;
    assert_eq!(basic_info(&res)?.exponent_bits_per_sample, 5);
    decoder.decode(&res)?;

    Ok(())
}

#[test]
fn bit_depth_differs_from_pixels() -> TestResult {
    let sample = get_sample().to_rgb8();
    let info = ImageInfo::builder()
        .width(sample.width())
        .height(sample.height())
        .bits_per_sample(16)
        .build();
    let mut encoder = encoder_builder().build()?;
    let mut session = encoder.session(&info)?;
    session.add_frame(&EncoderFrame::new(sample.as_raw()))?;
    let res = session.finish()?;

    assert_eq!(basic_info(&res)?.bits_per_sample, 16);
    Ok(())
}

#[test]
fn multi_frames() -> TestResult {
    let sample = get_sample().to_rgb8();
    let info = ImageInfo::builder()
        .width(sample.width())
        .height(sample.height())
        .build();
    let mut encoder = encoder_builder().use_container(true).build()?;

    let frame = EncoderFrame::new(sample.as_raw())
        .endianness(Endianness::Native)
        .align(0);

    let mut session = encoder.session(&info)?;
    session.add_frame(&frame)?;
    session.add_frame(&frame)?;
    let result = session.finish()?;
    let decoder = decoder_builder().build()?;
    let _res = decoder.decode(&result)?;

    encoder.uses_original_profile = true;
    let mut session = encoder.jpeg_session()?;
    session.add_jpeg_frame(super::SAMPLE_JPEG)?;
    session.add_jpeg_frame(super::SAMPLE_JPEG)?;
    let result = session.finish()?;
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

    let result = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;
    _ = decoder.decode(&result)?;

    encoder.color_encoding = Some(ColorEncoding::LinearSrgbLuma);
    let result = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;
    _ = decoder.decode(&result)?;

    Ok(())
}

#[test]
fn initial_buffer() -> TestResult {
    let mut encoder = encoder_builder().init_buffer_size(0).build()?;
    let sample = get_sample();
    let (w, h) = (sample.width(), sample.height());
    let _ = encoder.encode(sample.to_rgb8().as_raw(), w, h)?;
    let _ = encoder.encode(sample.to_rgb16().as_raw(), w, h)?;
    let _ = encoder.encode(sample.to_rgb32f().as_raw(), w, h)?;
    Ok(())
}

#[test]
fn custom_color_encoding() -> TestResult {
    let mut encoder = encoder_builder().build()?;
    let sample = get_sample().to_rgb16();

    // scRGB: linear transfer with sRGB primaries and D65 white point.
    let custom_color_encoding = JxlColorEncoding {
        color_space: JxlColorSpace::Rgb,
        white_point: JxlWhitePoint::D65,
        white_point_xy: [0.0, 0.0],
        primaries: JxlPrimaries::SRgb,
        primaries_red_xy: [0.0, 0.0],
        primaries_green_xy: [0.0, 0.0],
        primaries_blue_xy: [0.0, 0.0],
        transfer_function: JxlTransferFunction::Linear,
        gamma: 0.0,
        rendering_intent: JxlRenderingIntent::Relative,
    };
    encoder.color_encoding = Some(ColorEncoding::Custom(custom_color_encoding));
    encoder.target_intensity = Some(1000.0);

    let result = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    let decoder = decoder_builder().build()?;
    let _res = decoder.decode(&result)?;

    Ok(())
}

#[test]
fn reuse_keeps_options_and_boxes() -> TestResult {
    let sample = get_sample().to_rgb8();
    let (w, h) = (sample.width(), sample.height());
    let mut encoder = encoder_builder().build()?;
    let plain = encoder.encode(sample.as_raw(), w, h)?;

    encoder.set_frame_option(JxlEncoderFrameSettingId::Modular, 1);
    let first = encoder.encode(sample.as_raw(), w, h)?;
    let second = encoder.encode(sample.as_raw(), w, h)?;
    assert_ne!(plain, first);
    assert_eq!(first, second);

    for _ in 0..2 {
        encoder.add_metadata(&Metadata::Exif(super::SAMPLE_EXIF), false);
        let res = encoder.encode(sample.as_raw(), w, h)?;
        assert!(box_types(&res)?.contains(b"Exif"));
    }

    encoder.add_metadata(&Metadata::Exif(super::SAMPLE_EXIF), false);
    assert!(encoder.encode::<u8>(&[], 0, 0).is_err());
    let res = encoder.encode(sample.as_raw(), w, h)?;
    assert!(box_types(&res)?.contains(b"Exif"));
    let res = encoder.encode(sample.as_raw(), w, h)?;
    assert!(!box_types(&res)?.contains(b"Exif"));

    encoder.set_frame_option(JxlEncoderFrameSettingId::Effort, 100);
    assert!(encoder.encode(sample.as_raw(), w, h).is_err());
    encoder.set_frame_option(JxlEncoderFrameSettingId::Effort, 3);
    assert_eq!(encoder.frame_settings().options.len(), 2);
    let _ = encoder.encode(sample.as_raw(), w, h)?;

    Ok(())
}

#[test]
fn session_frames() -> TestResult {
    let sample = get_sample().to_rgb8();
    let info = ImageInfo::builder()
        .width(sample.width())
        .height(sample.height())
        .build();
    let mut encoder = encoder_builder()
        .uses_original_profile(true)
        .use_box(true)
        .build()?;

    let mut session = encoder.session(&info)?;
    let lossless = FrameSettings {
        lossless: Some(true),
        ..session.frame_settings()
    };
    session.add_metadata(&Metadata::Xmp(super::SAMPLE_XMP), true)?;
    let mut data = session.take_output()?;
    session.add_frame(&EncoderFrame::new(sample.as_raw()))?;
    data.extend(session.take_output()?);
    session.add_frame(&EncoderFrame::new(sample.as_raw()).settings(&lossless))?;
    data.extend(session.finish()?);

    decoder_builder().build()?.decode(&data)?;

    let mut session = encoder.session(&info)?;
    session.add_frame(&EncoderFrame::new(sample.as_raw()))?;
    session.take_output()?;
    assert!(matches!(
        session.finish(),
        Err(EncodeError::InvalidState(_))
    ));

    Ok(())
}

#[test]
fn animation() -> TestResult {
    use crate::encode::Animation;

    let pixels = vec![100u8; 8 * 8 * 3];
    let info = ImageInfo::builder()
        .width(8)
        .height(8)
        .animation(Animation::builder().tps_numerator(10).build())
        .build();
    let mut encoder = encoder_builder().build()?;
    let mut session = encoder.session(&info)?;
    session.add_frame(&EncoderFrame::new(&pixels).duration(3).name("first"))?;
    session.add_frame(&EncoderFrame::new(&pixels).duration(5))?;
    let data = session.finish()?;

    assert_eq!(basic_info(&data)?.animation.tps_numerator, 10);
    let mut decoder = decoder_builder().build()?;
    let mut session = decoder.session(Events::FRAME)?;
    let mut input = data.as_slice();
    let mut frames = vec![];
    loop {
        match session.process(&mut input)? {
            Event::Frame(frame) => frames.push((frame.header.duration, frame.name)),
            Event::Success => break,
            _ => {}
        }
    }
    assert_eq!(frames, [(3, "first".to_string()), (5, String::new())]);

    let mut session = encoder.session(&info)?;
    assert!(matches!(
        session.add_frame(&EncoderFrame::new(&pixels).name("a\0b")),
        Err(EncodeError::BadInput)
    ));

    Ok(())
}

#[test]
fn bit_depth_from_image() -> TestResult {
    let pixels = vec![1023u16; 8 * 8 * 3];
    let info = ImageInfo::builder()
        .width(8)
        .height(8)
        .bits_per_sample(10)
        .build();
    let mut encoder = encoder_builder()
        .lossless(true)
        .uses_original_profile(true)
        .build()?;
    let decoder = decoder_builder().build()?;

    let mut session = encoder.session(&info)?;
    session.add_frame(&EncoderFrame::new(&pixels).bit_depth_from_image())?;
    let (_, out) = decoder.decode_with::<u16>(&session.finish()?)?;
    assert!(out.iter().all(|&v| v == u16::MAX));

    let mut session = encoder.session(&info)?;
    session.add_frame(&EncoderFrame::new(&pixels))?;
    let (_, out) = decoder.decode_with::<u16>(&session.finish()?)?;
    assert!(out.iter().all(|&v| v < 2048));

    Ok(())
}
