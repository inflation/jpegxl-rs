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
use pretty_assertions::assert_eq;
use testresult::TestResult;

use crate::{
    common::Endianness,
    decode::{Data, Metadata, PixelFormat, Pixels},
    decoder_builder,
};
use crate::{ResizableRunner, ThreadsRunner};

#[test]
fn default_pixel_formats() -> TestResult {
    let decoder = decoder_builder().icc_profile(true).build()?;
    for (sample, channels, bits) in [
        (super::SAMPLE_JXL, 4, 16),
        (super::SAMPLE_JXL_GRAY, 1, 16),
        (super::SAMPLE_JXL_2BIT, 3, 8),
    ] {
        let (metadata, pixels) = decoder.decode(sample)?;
        let len = match (bits, pixels) {
            (8, Pixels::Uint8(pixels)) => pixels.len(),
            (16, Pixels::Uint16(pixels)) => pixels.len(),
            _ => panic!("expected {bits}-bit pixels for the {channels}-channel fixture"),
        };
        assert_eq!(len, (metadata.width * metadata.height * channels) as usize);
        lcms2::Profile::new_icc(&metadata.icc_profile.expect("expected an ICC profile"))?;
    }
    Ok(())
}

#[test]
fn typed_pixels_are_independent_of_endianness() -> TestResult {
    let mut decoder = decoder_builder().build()?;
    let integers = decoder.decode_with::<u16>(super::SAMPLE_JXL)?.1;
    let half_precision = decoder.decode_with::<f16>(super::SAMPLE_JXL)?.1;
    let single_precision = decoder.decode_with::<f32>(super::SAMPLE_JXL)?.1;
    for endianness in [Endianness::Big, Endianness::Little] {
        decoder.pixel_format = Some(PixelFormat {
            endianness,
            ..PixelFormat::default()
        });
        assert_eq!(decoder.decode_with::<u16>(super::SAMPLE_JXL)?.1, integers);
        assert_eq!(
            decoder.decode_with::<f16>(super::SAMPLE_JXL)?.1,
            half_precision
        );
        assert_eq!(
            decoder.decode_with::<f32>(super::SAMPLE_JXL)?.1,
            single_precision
        );
    }
    Ok(())
}

#[test]
fn jpeg() -> TestResult {
    let decoder = decoder_builder().init_jpeg_buffer(512).build()?;

    let (_, data) = decoder.reconstruct(super::SAMPLE_JXL_JPEG)?;
    let Data::Jpeg(data) = data else {
        panic!("Failed to reconstruct");
    };

    assert_eq!(data, super::SAMPLE_JPEG);

    let (_, data) = decoder.reconstruct(super::SAMPLE_JXL)?;
    assert!(matches!(data, Data::Pixels(Pixels::Uint16(_))));

    Ok(())
}

#[test]
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

#[test]
fn cropped_layer_with_padded_rows() -> TestResult {
    use crate::decode::{Event, Events};
    use jpegxl_sys::common::types::JxlDataType;

    // A 4x3 canvas followed by a 1x2 RGB layer.
    let data = include_bytes!("../../../samples/cropped_layer.jxl");
    let mut decoder = decoder_builder().coalescing(false).build()?;
    let (metadata, expected) = decoder.decode_with::<f16>(data)?;
    assert_eq!((metadata.width, metadata.height), (1, 2));
    assert_eq!(expected.len(), 6);

    let format = PixelFormat {
        align: 7,
        ..PixelFormat::default()
    };
    decoder.pixel_format = Some(format);
    assert_eq!(decoder.decode_with::<f16>(data)?.1, expected);

    #[cfg(feature = "image")]
    {
        use crate::image::ToDynamic;
        let image = decoder.decode_to_image(data)?.unwrap();
        assert_eq!((image.width(), image.height()), (1, 2));
    }

    let mut session = decoder.session(Events::FULL_IMAGE)?;
    let mut input = data.as_slice();
    let mut last = None;
    loop {
        match session.process(&mut input)? {
            Event::NeedImageOutBuffer => {
                session.alloc_image_buffer(format, Some(JxlDataType::Float16))?;
            }
            Event::FullImage(image) => last = image,
            Event::Success => break,
            _ => {}
        }
    }
    let last = last.unwrap();
    assert_eq!(last.data.len(), 13); // two 6-byte rows and one byte of padding
    let Pixels::Float16(pixels) = last.into_pixels() else {
        panic!("expected f16 pixels")
    };
    assert_eq!(pixels, expected);
    Ok(())
}

#[test]
fn failed_setup_resets_applied_options() -> TestResult {
    let data = include_bytes!("../../../samples/cropped_layer.jxl");
    let mut decoder = decoder_builder()
        .coalescing(false)
        .desired_intensity_target(-1.0)
        .build()?;
    assert!(decoder.decode(data).is_err());
    decoder.coalescing = None;
    decoder.desired_intensity_target = None;
    let (_, expected) = decoder_builder().build()?.decode_with::<u8>(data)?;
    let (metadata, pixels) = decoder.decode_with::<u8>(data)?;
    assert_eq!(
        pixels.len(),
        metadata.width as usize * metadata.height as usize * 3
    );
    assert_eq!(pixels, expected);
    Ok(())
}

#[cfg(feature = "vendored")]
#[test]
fn compressed_icc_round_trip() -> TestResult {
    use crate::memory::{tests::BumpManager, MemoryManager};
    use jpegxl_sys::{
        common::types::JxlBool,
        metadata::compressed_icc::{JxlICCProfileDecode, JxlICCProfileEncode},
    };

    let decoder = decoder_builder().icc_profile(true).build()?;
    let (metadata, _) = decoder.decode(super::SAMPLE_JXL)?;
    let profile = metadata.icc_profile.unwrap();
    let arena = BumpManager::new(16 * 1024 * 1024);
    let manager = arena.manager();
    let mut compressed = std::ptr::null_mut();
    let mut decompressed = std::ptr::null_mut();
    let (mut compressed_len, mut decompressed_len) = (0, 0);
    // SAFETY: the arena owns all returned buffers through the comparison.
    unsafe {
        assert_eq!(
            JxlICCProfileEncode(
                &raw const manager,
                profile.as_ptr(),
                profile.len(),
                &raw mut compressed,
                &raw mut compressed_len
            ),
            JxlBool::True
        );
        assert_eq!(
            JxlICCProfileDecode(
                &raw const manager,
                compressed,
                compressed_len,
                &raw mut decompressed,
                &raw mut decompressed_len
            ),
            JxlBool::True
        );
        assert_eq!(
            profile.as_slice(),
            std::slice::from_raw_parts(decompressed, decompressed_len)
        );
    }
    Ok(())
}
