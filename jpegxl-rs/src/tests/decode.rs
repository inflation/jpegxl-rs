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

use std::io::Cursor;

use half::f16;
use image::ImageDecoder;
use pretty_assertions::assert_eq;
use testresult::TestResult;

use crate::{
    common::Endianness,
    decode::{Data, Metadata, PixelFormat, Pixels},
    decoder_builder, DecodeError,
};
#[cfg(feature = "threads")]
use crate::{ResizableRunner, ThreadsRunner};

#[test]
fn invalid() -> TestResult {
    let decoder = decoder_builder().build()?;

    assert!(matches!(
        decoder.decode(&[0x00, 0x00]),
        Err(DecodeError::InvalidInput)
    ));

    Ok(())
}

#[test]
fn simple() -> TestResult {
    let decoder = decoder_builder().icc_profile(true).build()?;

    let (
        Metadata {
            width,
            height,
            icc_profile,
            ..
        },
        data,
    ) = decoder.decode(super::SAMPLE_JXL)?;

    let Pixels::Uint16(data) = data else {
        return Err("Failed to decode".into());
    };

    assert_eq!(data.len(), (width * height * 4) as usize);
    // Check if icc profile is valid
    lcms2::Profile::new_icc(&icc_profile.expect("ICC profile not retrieved"))?;

    Ok(())
}

#[test]
fn sample_2bit() -> TestResult {
    let decoder = decoder_builder().build()?;

    let (Metadata { width, height, .. }, data) = decoder.decode(super::SAMPLE_JXL_2BIT)?;
    let Pixels::Uint8(data) = data else {
        return Err("Failed to decode".into());
    };
    assert_eq!(data.len(), (width * height * 3) as usize);

    Ok(())
}

#[test]
fn sample_gray() -> TestResult {
    let decoder = decoder_builder().build()?;

    let (Metadata { width, height, .. }, data) = decoder.decode(super::SAMPLE_JXL_GRAY)?;
    let Pixels::Uint16(data) = data else {
        return Err("Failed to decode".into());
    };
    assert_eq!(data.len(), (width * height) as usize);

    Ok(())
}

#[test]
fn pixel_types() -> TestResult {
    let mut decoder = decoder_builder().build()?;

    // Check different pixel types
    decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    decoder.decode_with::<u8>(super::SAMPLE_JXL)?;
    decoder.decode_with::<u16>(super::SAMPLE_JXL)?;
    decoder.decode_with::<f16>(super::SAMPLE_JXL)?;

    // Check endianness
    decoder.pixel_format = Some(PixelFormat {
        endianness: Endianness::Big,
        ..Default::default()
    });
    decoder.decode_with::<u16>(super::SAMPLE_JXL)?;
    decoder.decode_with::<f16>(super::SAMPLE_JXL)?;

    decoder.pixel_format = Some(PixelFormat {
        endianness: Endianness::Little,
        ..Default::default()
    });
    decoder.decode_with::<f16>(super::SAMPLE_JXL)?;

    Ok(())
}

#[test]
fn jpeg() -> TestResult {
    let decoder = decoder_builder().init_jpeg_buffer(512).build()?;

    let (_, data) = decoder.reconstruct(super::SAMPLE_JXL_JPEG)?;
    let Data::Jpeg(data) = data else {
        return Err("Failed to reconstruct".into());
    };

    let jpeg = image::codecs::jpeg::JpegDecoder::new(Cursor::new(data))?;
    let mut v = vec![0; jpeg.total_bytes().try_into().unwrap()];
    jpeg.read_image(&mut v)?;

    let (_, data) = decoder.reconstruct(super::SAMPLE_JXL)?;
    assert!(matches!(data, Data::Pixels(Pixels::Uint16(_))));

    Ok(())
}

#[test]
#[cfg(feature = "threads")]
fn builder() -> TestResult {
    use crate::decode::ProgressiveDetail;

    let threads_runner = ThreadsRunner::default();
    let resizable_runner = ResizableRunner::default();
    let mut decoder = decoder_builder()
        .pixel_format(PixelFormat {
            num_channels: 3,
            endianness: Endianness::Big,
            align: 10,
        })
        .desired_intensity_target(0.5)
        .coalescing(false)
        .progressive_detail(ProgressiveDetail::Passes)
        .render_spotcolors(false)
        .decompress(true)
        .unpremul_alpha(true)
        .skip_reorientation(true)
        .parallel_runner(&resizable_runner)
        .build()?;

    let (Metadata { width, height, .. }, data) = decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    assert_eq!(data.len(), (width * height * 3) as usize);

    // Set options after creating decoder
    decoder.pixel_format = Some(PixelFormat {
        num_channels: 4,
        endianness: Endianness::Little,
        ..PixelFormat::default()
    });
    decoder.skip_reorientation = Some(true);
    decoder.parallel_runner = Some(&threads_runner);

    decoder.decode(super::SAMPLE_JXL)?;
    let (Metadata { width, height, .. }, data) = decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    assert_eq!(data.len(), (width * height * 4) as usize);

    Ok(())
}
