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

    macro_rules! jxl_dec_events {
        ( $( $x: expr ),* ) => {
            {
                let mut tmp = 0;
                $(
                    tmp |= $x as i32;
                )*
                tmp
            }
        };
    }

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
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_bindings_version() {
        unsafe {
            assert_eq!(JxlDecoderVersion(), 11001);
            assert_eq!(JxlEncoderVersion(), 11001);
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    unsafe fn decode(decoder: *mut JxlDecoder, sample: &[u8]) {
        use JxlDecoderStatus::{
            BasicInfo, Error, FullImage, NeedImageOutBuffer, NeedMoreInput, Success,
        };

        // Stop after getting the basic info and decoding the image
        let mut status = JxlDecoderSubscribeEvents(
            decoder,
            jxl_dec_events!(JxlDecoderStatus::BasicInfo, JxlDecoderStatus::FullImage),
        );
        jxl_dec_assert!(status, "Subscribe Events");

        // Read everything in memory
        let signature = JxlSignatureCheck(sample.as_ptr(), 2);
        assert_eq!(signature, JxlSignature::Codestream);

        let next_in = sample.as_ptr();
        let avail_in = sample.len();

        let pixel_format = JxlPixelFormat {
            num_channels: 3,
            data_type: JxlDataType::Uint8,
            endianness: JxlEndianness::Native,
            align: 0,
        };

        let mut basic_info;
        let mut buffer: Vec<f32> = Vec::new();
        let mut x_size = 0;
        let mut y_size = 0;

        status = JxlDecoderSetInput(decoder, next_in, avail_in);
        jxl_dec_assert!(status, "Set input");

        loop {
            status = JxlDecoderProcessInput(decoder);

            match status {
                Error => panic!("Decoder error!"),
                NeedMoreInput => {
                    panic!("Error, already provided all input")
                }

                // Get the basic info
                BasicInfo => {
                    basic_info = {
                        let mut info = MaybeUninit::uninit();
                        status = JxlDecoderGetBasicInfo(decoder, info.as_mut_ptr());
                        jxl_dec_assert!(status, "BasicInfo");
                        info.assume_init()
                    };

                    x_size = basic_info.xsize;
                    y_size = basic_info.ysize;
                    assert_eq!(basic_info.xsize, 40);
                    assert_eq!(basic_info.ysize, 50);
                }

                // Get the output buffer
                NeedImageOutBuffer => {
                    let mut size = 0;
                    status = JxlDecoderImageOutBufferSize(
                        decoder,
                        &raw const pixel_format,
                        &raw mut size,
                    );
                    jxl_dec_assert!(status, "BufferSize");

                    buffer.resize(size, 0f32);
                    status = JxlDecoderSetImageOutBuffer(
                        decoder,
                        &raw const pixel_format,
                        buffer.as_mut_ptr().cast(),
                        size,
                    );
                    jxl_dec_assert!(status, "SetBuffer");
                }

                FullImage => {}
                Success => {
                    assert_eq!(buffer.len(), (x_size * y_size * 3) as usize);
                    return;
                }
                _ => panic!("Unknown decoder status: {status:#?}"),
            }
        }
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_bindings_decoding() {
        unsafe {
            let dec = JxlDecoderCreate(ptr::null()); // Default memory manager
            assert!(!dec.is_null());

            // Simple single thread runner
            decode(dec, SAMPLE_JXL);

            JxlDecoderDestroy(dec);
        }
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_bindings_thread_pool() {
        unsafe {
            let runner = JxlThreadParallelRunnerCreate(
                std::ptr::null(),
                JxlThreadParallelRunnerDefaultNumWorkerThreads(),
            );

            let dec = JxlDecoderCreate(ptr::null()); // Default memory manager
            assert!(!dec.is_null());

            // Parallel multi-thread runner
            let status = JxlDecoderSetParallelRunner(dec, JxlThreadParallelRunner, runner);
            jxl_dec_assert!(status, "Set Parallel Runner");

            decode(dec, SAMPLE_JXL);

            JxlDecoderDestroy(dec);
            JxlThreadParallelRunnerDestroy(runner);
        }
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_bindings_resizable() {
        use JxlDecoderStatus::{
            BasicInfo, Error, FullImage, NeedImageOutBuffer, NeedMoreInput, Success,
        };

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

            // Stop after getting the basic info and decoding the image
            let mut status = JxlDecoderSubscribeEvents(
                dec,
                jxl_dec_events!(JxlDecoderStatus::BasicInfo, JxlDecoderStatus::FullImage),
            );
            jxl_dec_assert!(status, "Subscribe Events");

            // Read everything in memory
            let signature = JxlSignatureCheck(SAMPLE_JXL.as_ptr(), 2);
            assert_eq!(signature, JxlSignature::Codestream);

            let next_in = SAMPLE_JXL.as_ptr();
            let avail_in = SAMPLE_JXL.len();

            let pixel_format = JxlPixelFormat {
                num_channels: 3,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::Native,
                align: 0,
            };

            let mut basic_info;
            let mut buffer: Vec<f32> = Vec::new();
            let mut x_size = 0;
            let mut y_size = 0;

            status = JxlDecoderSetInput(dec, next_in, avail_in);
            jxl_dec_assert!(status, "Set input");

            loop {
                status = JxlDecoderProcessInput(dec);

                match status {
                    Error => panic!("Decoder error!"),
                    NeedMoreInput => {
                        panic!("Error, already provided all input")
                    }

                    // Get the basic info
                    BasicInfo => {
                        basic_info = {
                            let mut info = MaybeUninit::uninit();
                            status = JxlDecoderGetBasicInfo(dec, info.as_mut_ptr());
                            jxl_dec_assert!(status, "BasicInfo");
                            info.assume_init()
                        };
                        x_size = basic_info.xsize;
                        y_size = basic_info.ysize;

                        let num_threads = JxlResizableParallelRunnerSuggestThreads(
                            u64::from(x_size),
                            u64::from(y_size),
                        );
                        JxlResizableParallelRunnerSetThreads(runner, num_threads as usize);

                        assert_eq!(basic_info.xsize, 40);
                        assert_eq!(basic_info.ysize, 50);
                    }

                    // Get the output buffer
                    NeedImageOutBuffer => {
                        let mut size = 0;
                        status = JxlDecoderImageOutBufferSize(
                            dec,
                            &raw const pixel_format,
                            &raw mut size,
                        );
                        jxl_dec_assert!(status, "BufferSize");

                        buffer.resize(size, 0f32);
                        status = JxlDecoderSetImageOutBuffer(
                            dec,
                            &raw const pixel_format,
                            buffer.as_mut_ptr().cast(),
                            size,
                        );
                        jxl_dec_assert!(status, "SetBuffer");
                    }

                    FullImage => {}
                    Success => {
                        assert_eq!(buffer.len(), (x_size * y_size * 3) as usize);
                        break;
                    }
                    _ => panic!("Unknown decoder status: {status:#?}"),
                }
            }

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
                let mut basic_info = MaybeUninit::uninit();
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

            decode(dec, &output);

            JxlDecoderDestroy(dec);
            JxlThreadParallelRunnerDestroy(runner);
        }
    }
}
