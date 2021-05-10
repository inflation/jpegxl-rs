/*
This file is part of jpegxl-rs.

jpegxl-rs is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-rs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Decoder of JPEG XL format

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::*;
use std::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr::null,
};

use crate::{
    common::{Endianness, PixelType},
    errors::{check_dec_status, DecodeError},
    memory::JxlMemoryManager,
    parallel::JxlParallelRunner,
};

/// Basic Information
pub type BasicInfo = JxlBasicInfo;

/// Result of decoding
pub struct DecoderResult<T: PixelType> {
    /// Extra info
    pub info: ResultInfo,
    /// Decoded image data
    pub data: Vec<T>,
}

/// Extra info of the result
pub struct ResultInfo {
    /// Width of the image
    pub width: u32,
    /// Height of the image
    pub height: u32,
    /// Orientation
    pub orientation: JxlOrientation,
    /// Number of color channels per pixel
    pub num_channels: u32,
    /// ICC color profile
    pub icc_profile: Vec<u8>,
}

/// JPEG XL Decoder
pub struct JxlDecoder<'a> {
    /// Opaque pointer to the underlying decoder
    dec: *mut jpegxl_sys::JxlDecoder,

    /// Decoder options
    options: DecoderOptions,

    /// Parallel Runner
    parallel_runner: Option<&'a dyn JxlParallelRunner>,
}

union Data<T> {
    pixels: ManuallyDrop<Vec<T>>,
    jpeg: ManuallyDrop<Vec<u8>>,
}

impl<'a> JxlDecoder<'a> {
    fn new(
        options: DecoderOptions,
        memory_manager: Option<jpegxl_sys::JxlMemoryManager>,
        parallel_runner: Option<&'a dyn JxlParallelRunner>,
    ) -> Result<Self, DecodeError> {
        let dec = unsafe {
            memory_manager.map_or_else(
                || JxlDecoderCreate(null()),
                |memory_manager| JxlDecoderCreate(&memory_manager),
            )
        };

        if dec.is_null() {
            return Err(DecodeError::CannotCreateDecoder);
        }

        Ok(Self {
            dec,
            options,
            parallel_runner,
        })
    }

    /// Set number of channels
    pub fn set_num_channels(&mut self, value: u32) {
        self.options.num_channels = Some(value);
    }

    /// Set pixel endianness
    pub fn set_endianness(&mut self, value: Endianness) {
        self.options.endianness = Some(value);
    }

    /// Set pixel scanlines alignment
    pub fn set_align(&mut self, value: usize) {
        self.options.align = Some(value);
    }

    /// Keep original orientation or not. Default: `false`
    ///
    /// Note: Set `true` could make decoding faster, but you need to manually transform the result with the correct
    /// orientation.
    pub fn keep_orientation(&mut self, value: bool) {
        self.options.keep_orientation = value;
    }

    /// Decode a JPEG XL image
    ///
    /// Currently only support RGB(A)8/16/32 encoded static image. Other info are discarded.
    /// # Errors
    /// Return a [`DecodeError`] when internal decoder fails
    pub fn decode<T: PixelType>(&self, data: &[u8]) -> Result<DecoderResult<T>, DecodeError> {
        let (info, data) = self.decode_internal(data, false)?;
        Ok(DecoderResult {
            info,
            data: unsafe { ManuallyDrop::into_inner(data.pixels) },
        })
    }

    /// Decode a JPEG XL image and reconstruct JPEG data
    ///
    /// Currently only support RGB(A)8/16/32 encoded static image. Other info are discarded.
    /// # Errors
    /// Return a [`DecodeError`] when internal decoder fails
    pub fn decode_jpeg(&self, data: &[u8]) -> Result<DecoderResult<u8>, DecodeError> {
        let (info, data) = self.decode_internal::<u8>(data, true)?;
        Ok(DecoderResult {
            info,
            data: unsafe { ManuallyDrop::into_inner(data.jpeg) },
        })
    }

    fn decode_internal<T: PixelType>(
        &self,
        data: &[u8],
        reconstruct_jpeg: bool,
    ) -> Result<(ResultInfo, Data<T>), DecodeError> {
        let mut basic_info = MaybeUninit::uninit();
        let mut pixel_format = MaybeUninit::uninit();

        let mut icc_profile = MaybeUninit::uninit();
        let mut buffer = MaybeUninit::uninit();
        let mut jpeg_buffer = MaybeUninit::uninit();

        let mut jpeg_reconstructed = false;

        if reconstruct_jpeg {
            jpeg_buffer = MaybeUninit::new(vec![0; self.options.init_jpeg_buffer]);
        }

        self.setup_decoder(reconstruct_jpeg)?;

        let next_in = data.as_ptr();
        let avail_in = std::mem::size_of_val(data) as _;

        check_dec_status(
            unsafe { JxlDecoderSetInput(self.dec, next_in, avail_in) },
            "Set input",
        )?;

        let mut status;
        loop {
            use JxlDecoderStatus::*;

            status = unsafe { JxlDecoderProcessInput(self.dec) };

            match status {
                Error => return Err(DecodeError::GenericError("Process input")),

                // Get the basic info
                BasicInfo => {
                    self.get_basic_info::<T>(basic_info.as_mut_ptr(), pixel_format.as_mut_ptr())?;
                }

                // Get color encoding
                ColorEncoding => {
                    icc_profile =
                        MaybeUninit::new(self.get_icc_profile(unsafe { &*pixel_format.as_ptr() })?)
                }

                // Get JPEG reconstruction buffer
                JpegReconstruction => {
                    let jpeg_buffer = unsafe { &mut *jpeg_buffer.as_mut_ptr() };
                    jpeg_reconstructed = true;

                    check_dec_status(
                        unsafe {
                            JxlDecoderSetJPEGBuffer(
                                self.dec,
                                jpeg_buffer.as_mut_ptr(),
                                jpeg_buffer.len(),
                            )
                        },
                        "In JPEG reconstruction event",
                    )?;
                }

                // JPEG buffer need more space
                JpegNeedMoreOutput => {
                    let need_to_write = unsafe { JxlDecoderReleaseJPEGBuffer(self.dec) };

                    let jpeg_buffer = unsafe { &mut *jpeg_buffer.as_mut_ptr() };
                    let old_len = jpeg_buffer.len();
                    jpeg_buffer.resize(old_len + need_to_write, 0);
                    check_dec_status(
                        unsafe {
                            JxlDecoderSetJPEGBuffer(
                                self.dec,
                                jpeg_buffer.as_mut_ptr(),
                                jpeg_buffer.len(),
                            )
                        },
                        "In JPEG need more output event, set without releasing",
                    )?;
                }

                // Get the output buffer
                NeedImageOutBuffer => {
                    buffer = MaybeUninit::new(self.output(unsafe { &*pixel_format.as_ptr() })?);
                }

                FullImage => continue,
                Success => {
                    if reconstruct_jpeg {
                        if !jpeg_reconstructed {
                            return Err(DecodeError::CannotReconstruct);
                        }

                        let remaining = unsafe { JxlDecoderReleaseJPEGBuffer(self.dec) };

                        let jpeg_buffer = unsafe { &mut *jpeg_buffer.as_mut_ptr() };
                        jpeg_buffer.truncate(jpeg_buffer.len() - remaining);
                        jpeg_buffer.shrink_to_fit();
                    }

                    unsafe { JxlDecoderReset(self.dec) };

                    let info = unsafe { basic_info.assume_init() };
                    return Ok((
                        ResultInfo {
                            width: info.xsize,
                            height: info.ysize,
                            orientation: info.orientation,
                            num_channels: unsafe { pixel_format.assume_init().num_channels },
                            icc_profile: unsafe { icc_profile.assume_init() },
                        },
                        if reconstruct_jpeg {
                            Data {
                                jpeg: unsafe { ManuallyDrop::new(jpeg_buffer.assume_init()) },
                            }
                        } else {
                            Data {
                                pixels: unsafe { ManuallyDrop::new(buffer.assume_init()) },
                            }
                        },
                    ));
                }
                _ => return Err(DecodeError::UnknownStatus(status)),
            }
        }
    }

    fn setup_decoder(&self, reconstruct_jpeg: bool) -> Result<(), DecodeError> {
        if let Some(runner) = self.parallel_runner {
            check_dec_status(
                unsafe {
                    JxlDecoderSetParallelRunner(self.dec, runner.runner(), runner.as_opaque_ptr())
                },
                "Set parallel runner",
            )?
        }

        let events = {
            use JxlDecoderStatus::*;

            let mut events = jxl_dec_events!(BasicInfo, ColorEncoding, FullImage);

            if reconstruct_jpeg {
                events |= JpegReconstruction as i32;
            }

            events
        };
        check_dec_status(
            unsafe { JxlDecoderSubscribeEvents(self.dec, events) },
            "Subscribe events",
        )?;

        check_dec_status(
            unsafe { JxlDecoderSetKeepOrientation(self.dec, self.options.keep_orientation) },
            "Set if keep orientation",
        )?;

        Ok(())
    }

    fn get_basic_info<T: PixelType>(
        &self,
        basic_info: *mut JxlBasicInfo,
        pixel_format: *mut JxlPixelFormat,
    ) -> Result<(), DecodeError> {
        unsafe {
            check_dec_status(
                JxlDecoderGetBasicInfo(self.dec, basic_info),
                "Get basic info",
            )?;
        }

        unsafe {
            check_dec_status(
                JxlDecoderDefaultPixelFormat(self.dec, pixel_format),
                "Get default pixel format",
            )?;
        }

        let format = unsafe { &*pixel_format };

        let num_channels = self.options.num_channels.unwrap_or(format.num_channels);
        let endianness = self.options.endianness.unwrap_or(format.endianness);
        let align = self.options.align.unwrap_or(format.align);

        unsafe {
            *pixel_format = JxlPixelFormat {
                num_channels,
                data_type: T::pixel_type(),
                endianness,
                align,
            };
        }

        Ok(())
    }

    fn get_icc_profile(&self, format: &JxlPixelFormat) -> Result<Vec<u8>, DecodeError> {
        let mut icc_size = 0;

        check_dec_status(
            unsafe {
                JxlDecoderGetICCProfileSize(
                    self.dec,
                    format,
                    JxlColorProfileTarget::Data,
                    &mut icc_size,
                )
            },
            "Get ICC profile size",
        )?;

        let mut icc_profile = vec![0; icc_size];

        check_dec_status(
            unsafe {
                JxlDecoderGetColorAsICCProfile(
                    self.dec,
                    format,
                    JxlColorProfileTarget::Data,
                    icc_profile.as_mut_ptr(),
                    icc_size,
                )
            },
            "Get ICC profile",
        )?;

        icc_profile.shrink_to_fit();

        Ok(icc_profile)
    }

    fn output<T: PixelType>(&self, pixel_format: &JxlPixelFormat) -> Result<Vec<T>, DecodeError> {
        let mut size = 0;
        check_dec_status(
            unsafe { JxlDecoderImageOutBufferSize(self.dec, pixel_format, &mut size) },
            "Get output buffer size",
        )?;

        let mut buffer = vec![T::default(); size];
        check_dec_status(
            unsafe {
                JxlDecoderSetImageOutBuffer(
                    self.dec,
                    pixel_format,
                    buffer.as_mut_ptr().cast(),
                    size,
                )
            },
            "Set output buffer",
        )?;

        buffer.shrink_to_fit();

        Ok(buffer)
    }
}

impl<'a> Drop for JxlDecoder<'a> {
    fn drop(&mut self) {
        unsafe { JxlDecoderDestroy(self.dec) };
    }
}

#[derive(Clone)]
struct DecoderOptions {
    num_channels: Option<u32>,
    endianness: Option<JxlEndianness>,
    align: Option<usize>,
    keep_orientation: bool,
    init_jpeg_buffer: usize,
}

/// Builder for [`JxlDecoder`]
pub struct JxlDecoderBuilder<'a> {
    options: DecoderOptions,
    memory_manager: Option<&'a dyn JxlMemoryManager>,
    parallel_runner: Option<&'a dyn JxlParallelRunner>,
}

impl<'a> JxlDecoderBuilder<'a> {
    /// Set number of channels for returned result
    pub fn num_channels(&mut self, num: u32) -> &mut Self {
        self.options.num_channels = Some(num);
        self
    }

    /// Set endianness for returned result
    pub fn endian(&mut self, endian: JxlEndianness) -> &mut Self {
        self.options.endianness = Some(endian);
        self
    }

    /// Set align for returned result
    pub fn align(&mut self, align: usize) -> &mut Self {
        self.options.align = Some(align);
        self
    }

    /// Set initial buffer for JPEG reconstruction.
    /// Larger one could be faster with fewer allocations
    ///
    /// Default: 1 MiB
    pub fn init_jpeg_buffer(&mut self, value: usize) -> &mut Self {
        self.options.init_jpeg_buffer = value;
        self
    }

    /// Keep orientation or not
    pub fn keep_orientation(&mut self, value: bool) -> &mut Self {
        self.options.keep_orientation = value;
        self
    }

    /// Set memory manager
    pub fn memory_manager(&mut self, memory_manager: &'a dyn JxlMemoryManager) -> &mut Self {
        self.memory_manager = Some(memory_manager);
        self
    }

    /// Set parallel runner
    pub fn parallel_runner(&mut self, parallel_runner: &'a dyn JxlParallelRunner) -> &mut Self {
        self.parallel_runner = Some(parallel_runner);
        self
    }

    /// Consume the builder and get the decoder
    /// # Errors
    /// Return [`DecodeError::CannotCreateDecoder`] if it fails to create the decoder.
    pub fn build(&mut self) -> Result<JxlDecoder<'a>, DecodeError> {
        JxlDecoder::new(
            self.options.clone(),
            self.memory_manager.map(|m| m.to_manager()),
            self.parallel_runner,
        )
    }
}

/// Return a [`JxlDecoderBuilder`] with default settings
#[must_use]
pub fn decoder_builder<'a>() -> JxlDecoderBuilder<'a> {
    JxlDecoderBuilder {
        options: DecoderOptions {
            num_channels: None,
            endianness: None,
            align: None,
            keep_orientation: false,
            init_jpeg_buffer: 1024, // 1 KiB
        },
        memory_manager: None,
        parallel_runner: None,
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    use crate::ThreadsRunner;
    use image::ImageDecoder;

    #[test]
    fn test_decode() -> Result<(), Box<dyn Error>> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mm = crate::memory::tests::BumpManager::<{ 1024 * 5 }>::default();
        let parallel_runner = ThreadsRunner::new(Some(&mm.to_manager()), None).unwrap();
        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .memory_manager(&mm)
            .build()?;

        let DecoderResult { info, data, .. } = decoder.decode::<u8>(&sample)?;

        assert_eq!(data.len(), (info.width * info.height * 4) as usize);

        // Check if icc profile is valid
        qcms::Profile::new_from_slice(&info.icc_profile).ok_or("Failed to retrieve ICC profile")?;

        Ok(())
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_decode_container() -> Result<(), Box<dyn Error>> {
        let sample = std::fs::read("test/sample_jpg.jxl")?;
        let parallel_runner = ThreadsRunner::default();
        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .init_jpeg_buffer(512)
            .build()?;

        let DecoderResult { data, .. } = decoder.decode_jpeg(&sample)?;

        let jpeg = image::codecs::jpeg::JpegDecoder::new(data.as_slice())?;
        let mut v = vec![0; jpeg.total_bytes() as usize];
        jpeg.read_image(&mut v)?;

        let sample = std::fs::read("test/sample.jxl")?;
        assert!(matches!(
            decoder.decode_jpeg(&sample),
            Err(DecodeError::CannotReconstruct)
        ));

        Ok(())
    }

    #[test]
    fn test_decoder_builder() -> Result<(), Box<dyn Error>> {
        let sample = std::fs::read("test/sample.jxl")?;
        let parallel_runner = ThreadsRunner::default();
        let mut decoder = decoder_builder()
            .num_channels(3)
            .endian(JxlEndianness::Big)
            .align(2)
            .keep_orientation(true)
            .parallel_runner(&parallel_runner)
            .build()?;

        let DecoderResult { info, data, .. } = decoder.decode::<u8>(&sample)?;

        assert_eq!(data.len(), (info.width * info.height * 3) as usize);

        decoder.set_align(0);
        decoder.set_endianness(JxlEndianness::Native);
        decoder.set_num_channels(4);
        decoder.keep_orientation(true);

        decoder.decode::<u8>(&sample)?;

        Ok(())
    }

    #[test]
    fn test_invalid_data() -> Result<(), DecodeError> {
        let sample = Vec::new();

        let decoder = decoder_builder().build()?;
        assert!(matches!(
            decoder.decode::<u8>(&sample),
            Err(DecodeError::UnknownStatus(JxlDecoderStatus::NeedMoreInput))
        ));

        Ok(())
    }
}
