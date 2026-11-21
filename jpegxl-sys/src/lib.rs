/*
This file is part of jpegxl-sys.

jpegxl-sys is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-sys is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-sys.  If not, see <https://www.gnu.org/licenses/>.
*/

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod decode;

pub mod color;
pub mod common;
pub mod encoder;
pub mod metadata;
pub mod threads;

#[cfg(test)]
mod test {
    use crate::{
        common::types::*,
        decode::*,
        encoder::encode::*,
        threads::thread_parallel_runner::{
            JxlThreadParallelRunner, JxlThreadParallelRunnerCreate,
            JxlThreadParallelRunnerDefaultNumWorkerThreads, JxlThreadParallelRunnerDestroy,
        },
    };

    use std::{mem::MaybeUninit, ptr};

    use pretty_assertions::assert_eq;

    const SAMPLE_PNG: &[u8] = include_bytes!("../../samples/sample.png");
    const SAMPLE_JXL: &[u8] = include_bytes!("../../samples/sample.jxl");

    macro_rules! jxl_dec_assert {
        ($val:expr, $desc:expr) => {
            if $val != JxlDecoderStatus::Success as _ {
                panic!("Decoder error by: {:#?}, in {}", $val, $desc)
            }
        };
    }

    macro_rules! jxl_enc_assert {
        ($val:expr, $desc:expr) => {
            if $val != JxlEncoderStatus::Success as _ {
                panic!("Encoder error by: {:#?}, in {}", $val, $desc)
            }
        };
    }

    #[test]
    fn default_cms_is_linked() {
        // SAFETY: libjxl returns a pointer to its static CMS interface.
        assert!(!unsafe { crate::color::cms::JxlGetDefaultCms() }.is_null());
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_bindings_version() {
        unsafe {
            for version in [JxlDecoderVersion(), JxlEncoderVersion()] {
                if cfg!(feature = "vendored") {
                    assert_eq!(version, 13000);
                } else {
                    assert!(version >= 13000, "unsupported libjxl version {version}");
                }
            }
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    unsafe fn decode(decoder: *mut JxlDecoder, sample: &[u8], on_size: impl FnOnce(u32, u32)) {
        use JxlDecoderStatus::{BasicInfo, FullImage, NeedImageOutBuffer, Success};

        assert_eq!(
            JxlSignatureCheck(sample.as_ptr(), sample.len()),
            JxlSignature::Codestream
        );
        let events = BasicInfo as i32 | FullImage as i32;
        assert_eq!(JxlDecoderSubscribeEvents(decoder, events), Success);
        assert_eq!(
            JxlDecoderSetInput(decoder, sample.as_ptr(), sample.len()),
            Success
        );
        assert_eq!(JxlDecoderProcessInput(decoder), BasicInfo);

        let mut info = MaybeUninit::zeroed();
        assert_eq!(JxlDecoderGetBasicInfo(decoder, info.as_mut_ptr()), Success);
        let info = info.assume_init();
        assert_eq!((info.xsize, info.ysize), (40, 50));
        on_size(info.xsize, info.ysize);

        let format = JxlPixelFormat {
            num_channels: 3,
            data_type: JxlDataType::Uint8,
            endianness: JxlEndianness::Native,
            align: 0,
        };
        assert_eq!(JxlDecoderProcessInput(decoder), NeedImageOutBuffer);
        let mut size = 0;
        assert_eq!(
            JxlDecoderImageOutBufferSize(decoder, &raw const format, &raw mut size),
            Success
        );
        assert_eq!(
            size,
            (info.xsize * info.ysize * format.num_channels) as usize
        );
        let mut pixels = vec![0u8; size];
        assert_eq!(
            JxlDecoderSetImageOutBuffer(
                decoder,
                &raw const format,
                pixels.as_mut_ptr().cast(),
                size
            ),
            Success
        );
        assert_eq!(JxlDecoderProcessInput(decoder), FullImage);
        assert_eq!(JxlDecoderProcessInput(decoder), Success);
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_bindings_decoding() {
        unsafe {
            let dec = JxlDecoderCreate(ptr::null()); // Default memory manager
            assert!(!dec.is_null());

            // Simple single thread runner
            decode(dec, SAMPLE_JXL, |_, _| {});

            JxlDecoderDestroy(dec);
        }
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_bindings_resizable() {
        use crate::threads::resizable_parallel_runner::{
            JxlResizableParallelRunner, JxlResizableParallelRunnerCreate,
            JxlResizableParallelRunnerDestroy, JxlResizableParallelRunnerSetThreads,
            JxlResizableParallelRunnerSuggestThreads,
        };

        unsafe {
            let runner = JxlResizableParallelRunnerCreate(std::ptr::null());

            let dec = JxlDecoderCreate(ptr::null()); // Default memory manager
            assert!(!dec.is_null());

            // Resizable parallel multi-thread runner
            let status = JxlDecoderSetParallelRunner(dec, JxlResizableParallelRunner, runner);
            jxl_dec_assert!(status, "Set Parallel Runner");

            decode(dec, SAMPLE_JXL, |width, height| {
                let threads = JxlResizableParallelRunnerSuggestThreads(width.into(), height.into());
                JxlResizableParallelRunnerSetThreads(runner, threads as usize);
            });

            JxlDecoderDestroy(dec);
            JxlResizableParallelRunnerDestroy(runner);
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn encode(pixels: &[u8], x_size: u32, ysize: u32) -> Vec<u8> {
        unsafe {
            let enc = JxlEncoderCreate(std::ptr::null());

            let runner = JxlThreadParallelRunnerCreate(
                std::ptr::null(),
                JxlThreadParallelRunnerDefaultNumWorkerThreads(),
            );

            let mut status = JxlEncoderSetParallelRunner(enc, JxlThreadParallelRunner, runner);
            jxl_enc_assert!(status, "Set Parallel Runner");

            let mut basic_info = {
                // libjxl leaves the reserved bytes untouched.
                let mut basic_info = MaybeUninit::zeroed();
                JxlEncoderInitBasicInfo(basic_info.as_mut_ptr());
                basic_info.assume_init()
            };
            basic_info.xsize = x_size;
            basic_info.ysize = ysize;

            status = JxlEncoderSetBasicInfo(enc, &raw const basic_info);
            jxl_enc_assert!(status, "Set Basic Info");

            let pixel_format = JxlPixelFormat {
                num_channels: 3,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::Native,
                align: 0,
            };
            let mut color_encoding = MaybeUninit::uninit();
            JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), false.into());
            status = JxlEncoderSetColorEncoding(enc, color_encoding.as_ptr());
            jxl_enc_assert!(status, "Set Color Encoding");

            status = JxlEncoderAddImageFrame(
                JxlEncoderFrameSettingsCreate(enc, std::ptr::null()),
                &raw const pixel_format,
                pixels.as_ptr().cast_mut().cast(),
                pixels.len(),
            );
            jxl_enc_assert!(status, "Add Image Frame");

            JxlEncoderCloseInput(enc);

            let chunk_size = 1024 * 512; // 512 KB is a good initial value
            let mut buffer = vec![0u8; chunk_size];
            let mut next_out = buffer.as_mut_ptr();
            let mut avail_out = chunk_size;

            loop {
                status = JxlEncoderProcessOutput(
                    enc,
                    std::ptr::addr_of_mut!(next_out),
                    &raw mut avail_out,
                );

                if status != JxlEncoderStatus::NeedMoreOutput {
                    break;
                }

                let offset = next_out as usize - buffer.as_ptr() as usize;
                buffer.resize(buffer.len() * 2, 0);
                next_out = buffer.as_mut_ptr().add(offset);
                avail_out = buffer.len() - offset;
            }
            buffer.truncate(next_out as usize - buffer.as_ptr() as usize);
            jxl_enc_assert!(status, "Encoding");

            JxlEncoderDestroy(enc);
            JxlThreadParallelRunnerDestroy(runner);

            buffer
        }
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_bindings_encoding() {
        let img = image::load_from_memory_with_format(SAMPLE_PNG, image::ImageFormat::Png).unwrap();
        let image_buffer = img.into_rgb8();

        let output = encode(
            image_buffer.as_raw(),
            image_buffer.width(),
            image_buffer.height(),
        );

        unsafe {
            let runner = JxlThreadParallelRunnerCreate(
                std::ptr::null(),
                JxlThreadParallelRunnerDefaultNumWorkerThreads(),
            );

            let dec = JxlDecoderCreate(ptr::null()); // Default memory manager
            assert!(!dec.is_null());

            let status = JxlDecoderSetParallelRunner(dec, JxlThreadParallelRunner, runner);
            jxl_dec_assert!(status, "Set Parallel Runner");

            decode(dec, &output, |_, _| {});

            JxlDecoderDestroy(dec);
            JxlThreadParallelRunnerDestroy(runner);
        }
    }
}
