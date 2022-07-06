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

#![warn(clippy::pedantic)]

pub mod bindings;

pub use bindings::*;

macro_rules! trait_impl {
    ($x:ty, [$($struct_:ident ),*]) => {
        $(
            impl $x for $struct_ {}
        )*
    };
}

trait_impl!(NewUninit, [JxlBasicInfo, JxlPixelFormat, JxlColorEncoding]);

/// Convenient function to just return a block of memory.
/// You need to assign `basic_info.assume_init()` to use as a Rust struct after passing as a pointer.
/// # Examples:
/// ```
/// # use jpegxl_sys::*;
/// # unsafe {
/// # let decoder = JxlDecoderCreate(std::ptr::null());
/// let mut basic_info = JxlBasicInfo::new_uninit();
/// JxlDecoderGetBasicInfo(decoder, basic_info.as_mut_ptr());
/// let basic_info = basic_info.assume_init();
/// }
/// ```
pub trait NewUninit {
    #[inline]
    #[must_use]
    fn new_uninit() -> std::mem::MaybeUninit<Self>
    where
        Self: std::marker::Sized,
    {
        std::mem::MaybeUninit::<Self>::uninit()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parallel_runner::{
        JxlThreadParallelRunner, JxlThreadParallelRunnerCreate,
        JxlThreadParallelRunnerDefaultNumWorkerThreads, JxlThreadParallelRunnerDestroy,
    };
    use std::ptr;

    use image::io::Reader as ImageReader;
    use image::ImageError;

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
    fn test_bindings_version() {
        unsafe {
            assert_eq!(JxlDecoderVersion(), 6001);
            assert_eq!(JxlEncoderVersion(), 6001);
        }
    }

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
        assert_eq!(signature, JxlSignature::Codestream, "Signature");

        let next_in = sample.as_ptr();
        let avail_in = sample.len();

        let pixel_format = JxlPixelFormat {
            num_channels: 3,
            data_type: JxlDataType::Uint8,
            endianness: JxlEndianness::Native,
            align: 0,
        };

        let mut basic_info = JxlBasicInfo::new_uninit().assume_init();
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
                    status = JxlDecoderGetBasicInfo(decoder, &mut basic_info);
                    jxl_dec_assert!(status, "BasicInfo");
                    x_size = basic_info.xsize;
                    y_size = basic_info.ysize;
                    assert_eq!(basic_info.xsize, 40, "Width");
                    assert_eq!(basic_info.ysize, 50, "Height");
                }

                // Get the output buffer
                NeedImageOutBuffer => {
                    let mut size = 0;
                    status = JxlDecoderImageOutBufferSize(decoder, &pixel_format, &mut size);
                    jxl_dec_assert!(status, "BufferSize");

                    buffer.resize(size as usize, 0f32);
                    status = JxlDecoderSetImageOutBuffer(
                        decoder,
                        &pixel_format,
                        buffer.as_mut_ptr().cast(),
                        size,
                    );
                    jxl_dec_assert!(status, "SetBuffer");
                }

                FullImage => continue,
                Success => {
                    assert_eq!(buffer.len(), (x_size * y_size * 3) as usize);
                    return;
                }
                _ => panic!("Unknown decoder status: {:#?}", status),
            }
        }
    }

    #[test]
    fn test_bindings_decoding() {
        unsafe {
            let dec = JxlDecoderCreate(ptr::null()); // Default memory manager
            assert!(!dec.is_null());

            // Simple single thread runner
            let sample = std::fs::read("test/sample.jxl").unwrap();
            decode(dec, &sample);

            JxlDecoderDestroy(dec);
        }
    }

    #[test]
    #[cfg(feature = "threads")]
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

            let sample = std::fs::read("test/sample.jxl").unwrap();
            decode(dec, &sample);

            JxlDecoderDestroy(dec);
            JxlThreadParallelRunnerDestroy(runner);
        }
    }

    #[test]
    #[cfg(feature = "threads")]
    fn test_bindings_resizable() {
        use JxlDecoderStatus::{
            BasicInfo, Error, FullImage, NeedImageOutBuffer, NeedMoreInput, Success,
        };

        use crate::resizable_parallel_runner::{
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

            let sample = std::fs::read("test/sample.jxl").unwrap();

            // Stop after getting the basic info and decoding the image
            let mut status = JxlDecoderSubscribeEvents(
                dec,
                jxl_dec_events!(JxlDecoderStatus::BasicInfo, JxlDecoderStatus::FullImage),
            );
            jxl_dec_assert!(status, "Subscribe Events");

            // Read everything in memory
            let signature = JxlSignatureCheck(sample.as_ptr(), 2);
            assert_eq!(signature, JxlSignature::Codestream, "Signature");

            let next_in = sample.as_ptr();
            let avail_in = sample.len();

            let pixel_format = JxlPixelFormat {
                num_channels: 3,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::Native,
                align: 0,
            };

            let mut basic_info = JxlBasicInfo::new_uninit().assume_init();
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
                        status = JxlDecoderGetBasicInfo(dec, &mut basic_info);
                        jxl_dec_assert!(status, "BasicInfo");
                        x_size = basic_info.xsize;
                        y_size = basic_info.ysize;

                        let num_threads = JxlResizableParallelRunnerSuggestThreads(
                            u64::from(x_size),
                            u64::from(y_size),
                        );
                        JxlResizableParallelRunnerSetThreads(runner, num_threads as usize);

                        assert_eq!(basic_info.xsize, 40, "Width");
                        assert_eq!(basic_info.ysize, 50, "Height");
                    }

                    // Get the output buffer
                    NeedImageOutBuffer => {
                        let mut size = 0;
                        status = JxlDecoderImageOutBufferSize(dec, &pixel_format, &mut size);
                        jxl_dec_assert!(status, "BufferSize");

                        buffer.resize(size as usize, 0f32);
                        status = JxlDecoderSetImageOutBuffer(
                            dec,
                            &pixel_format,
                            buffer.as_mut_ptr().cast(),
                            size,
                        );
                        jxl_dec_assert!(status, "SetBuffer");
                    }

                    FullImage => continue,
                    Success => {
                        assert_eq!(buffer.len(), (x_size * y_size * 3) as usize);
                        break;
                    }
                    _ => panic!("Unknown decoder status: {:#?}", status),
                }
            }

            JxlDecoderDestroy(dec);
            JxlResizableParallelRunnerDestroy(runner);
        }
    }

    fn encode(pixels: &[u8], x_size: u32, ysize: u32) -> Vec<u8> {
        unsafe {
            let enc = JxlEncoderCreate(std::ptr::null());

            let runner = JxlThreadParallelRunnerCreate(
                std::ptr::null(),
                JxlThreadParallelRunnerDefaultNumWorkerThreads(),
            );

            let mut status = JxlEncoderSetParallelRunner(enc, JxlThreadParallelRunner, runner);
            jxl_enc_assert!(status, "Set Parallel Runner");

            let mut basic_info = JxlBasicInfo::new_uninit().assume_init();
            JxlEncoderInitBasicInfo(&mut basic_info);
            basic_info.xsize = x_size;
            basic_info.ysize = ysize;

            status = JxlEncoderSetBasicInfo(enc, &basic_info);
            jxl_enc_assert!(status, "Set Basic Info");

            let pixel_format = JxlPixelFormat {
                num_channels: 3,
                data_type: JxlDataType::Uint8,
                endianness: JxlEndianness::Native,
                align: 0,
            };
            let mut color_encoding = JxlColorEncoding::new_uninit().assume_init();
            JxlColorEncodingSetToSRGB(&mut color_encoding, false);
            status = JxlEncoderSetColorEncoding(enc, &color_encoding);
            jxl_enc_assert!(status, "Set Color Encoding");

            status = JxlEncoderAddImageFrame(
                JxlEncoderOptionsCreate(enc, std::ptr::null()),
                &pixel_format,
                pixels.as_ptr() as *mut std::ffi::c_void,
                pixels.len(),
            );
            jxl_enc_assert!(status, "Add Image Frame");

            let chunk_size = 1024 * 512; // 512 KB is a good initial value
            let mut buffer = vec![0u8; chunk_size];
            let mut next_out = buffer.as_mut_ptr();
            let mut avail_out = chunk_size;

            loop {
                status =
                    JxlEncoderProcessOutput(enc, std::ptr::addr_of_mut!(next_out), &mut avail_out);

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
    fn test_bindings_encoding() {
        || -> Result<(), ImageError> {
            let img = ImageReader::open("test/sample.png")?.decode()?;
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

            Ok(())
        }()
        .unwrap();
    }
}
