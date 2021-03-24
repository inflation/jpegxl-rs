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
use std::ptr::null;

use crate::{
    common::PixelType,
    errors::{check_dec_status, DecodeError},
    memory::JxlMemoryManager,
    parallel::JxlParallelRunner,
    Endianness,
};

/// Basic Information
pub type BasicInfo = JxlBasicInfo;

/// Result of decoding
pub struct DecoderResult<T: PixelType> {
    /// Extra info
    pub info: DecoderResultInfo,
    /// Decoded image data
    pub data: Vec<T>,
}

/// Extra info of the result
pub struct DecoderResultInfo {
    /// Width of the image
    pub width: u32,
    /// Height of the image
    pub height: u32,
    /// Orientation
    pub orientation: JxlOrientation,
    /// Has alpha channel
    pub has_alpha: bool,
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
    /// # Panics
    /// Only when underlying library events order is wrong
    pub fn decode<T: PixelType>(&mut self, data: &[u8]) -> Result<DecoderResult<T>, DecodeError> {
        let mut basic_info = None;
        let mut pixel_format = None;
        let mut icc_profile = Vec::new();
        let mut buffer = Vec::new();

        self.setup_decoder(data)?;

        let next_in = data.as_ptr();
        let avail_in = std::mem::size_of_val(data) as _;

        check_dec_status(unsafe { JxlDecoderSetInput(self.dec, next_in, avail_in) })?;

        let mut status;
        loop {
            use JxlDecoderStatus::*;

            status = unsafe { JxlDecoderProcessInput(self.dec) };

            match status {
                Error => return Err(DecodeError::GenericError),
                NeedMoreInput => return Err(DecodeError::NeedMoreInput),

                // Get the basic info
                BasicInfo => self.get_basic_info::<T>(&mut basic_info, &mut pixel_format)?,

                // Get color encoding
                ColorEncoding => {
                    self.get_icc_profile(pixel_format.as_ref().unwrap(), &mut icc_profile)?
                }

                // Get JPEG reconstruction buffer
                // FIXME: Upstream not implemented
                // JxlDecoderStatus_JXL_DEC_JPEG_RECONSTRUCTION => {
                //     check_dec_status(JxlDecoderSetJPEGBuffer(
                //         self.dec,
                //         buffer.as_mut_ptr() as _,
                //         buffer.len() as _,
                //     ))?;
                // }

                // // Need more output buffer
                // JxlDecoderStatus_JXL_DEC_JPEG_NEED_MORE_OUTPUT => {
                //     buffer.resize(buffer.len() * 2, T::default());
                //     check_dec_status(JxlDecoderSetJPEGBuffer(
                //         self.dec,
                //         buffer.as_mut_ptr() as _,
                //         buffer.len() as _,
                //     ))?;
                // }

                // Get the output buffer
                NeedImageOutBuffer => {
                    self.output(pixel_format.as_ref().unwrap(), &mut buffer)?;
                }

                FullImage => continue,
                Success => {
                    unsafe { JxlDecoderReset(self.dec) };
                    let info = basic_info.unwrap();
                    return Ok(DecoderResult {
                        info: DecoderResultInfo {
                            width: info.xsize,
                            height: info.ysize,
                            orientation: info.orientation,
                            has_alpha: info.alpha_bits != 0,
                            icc_profile,
                        },
                        data: buffer,
                    });
                }
                _ => return Err(DecodeError::UnknownStatus(status)),
            }
        }
    }

    fn setup_decoder(&mut self, data: &[u8]) -> Result<(), DecodeError> {
        let signature = unsafe { JxlSignatureCheck(data.as_ptr(), data.len()) };
        if signature != JxlSignature::Codestream && signature != JxlSignature::Container {
            return Err(DecodeError::InvalidFileFormat);
        }

        if let Some(runner) = self.parallel_runner {
            check_dec_status(unsafe {
                JxlDecoderSetParallelRunner(self.dec, Some(runner.runner()), runner.as_opaque_ptr())
            })?
        }

        let events = {
            use JxlDecoderStatus::*;

            let mut events = jxl_dec_events!(
                BasicInfo,
                ColorEncoding,
                if self.options.jpeg_reconstruction {
                    JpegReconstruction
                } else {
                    FullImage
                }
            );

            if self.options.need_preview {
                events |= PreviewImage as i32;
            }

            events
        };
        check_dec_status(unsafe { JxlDecoderSubscribeEvents(self.dec, events) })?;

        check_dec_status(unsafe {
            JxlDecoderSetKeepOrientation(self.dec, self.options.keep_orientation)
        })?;

        Ok(())
    }

    fn get_basic_info<T: PixelType>(
        &mut self,
        basic_info: &mut Option<JxlBasicInfo>,
        pixel_format: &mut Option<JxlPixelFormat>,
    ) -> Result<(), DecodeError> {
        *basic_info = Some(unsafe {
            let mut info = JxlBasicInfo::new_uninit();
            check_dec_status(JxlDecoderGetBasicInfo(self.dec, info.as_mut_ptr()))?;
            info.assume_init()
        });

        let format = unsafe {
            let mut format = JxlPixelFormat::new_uninit();
            check_dec_status(JxlDecoderDefaultPixelFormat(self.dec, format.as_mut_ptr()))?;
            format.assume_init()
        };

        let num_channels = self.options.num_channels.unwrap_or(format.num_channels);
        let endianness = self.options.endianness.unwrap_or(format.endianness);
        let align = self.options.align.unwrap_or(format.align);

        *pixel_format = Some(JxlPixelFormat {
            num_channels,
            data_type: T::pixel_type(),
            endianness,
            align,
        });

        Ok(())
    }

    fn get_icc_profile(
        &mut self,
        format: &JxlPixelFormat,
        icc_profile: &mut Vec<u8>,
    ) -> Result<(), DecodeError> {
        let mut icc_size = 0;

        check_dec_status(unsafe {
            JxlDecoderGetICCProfileSize(
                self.dec,
                format,
                JxlColorProfileTarget::Data,
                &mut icc_size,
            )
        })?;

        icc_profile.resize(icc_size, 0);

        check_dec_status(unsafe {
            JxlDecoderGetColorAsICCProfile(
                self.dec,
                format,
                JxlColorProfileTarget::Data,
                icc_profile.as_mut_ptr(),
                icc_size,
            )
        })?;

        Ok(())
    }

    fn output<T: PixelType>(
        &mut self,
        pixel_format: &JxlPixelFormat,
        buffer: &mut Vec<T>,
    ) -> Result<(), DecodeError> {
        let mut size = 0;
        check_dec_status(unsafe {
            JxlDecoderImageOutBufferSize(self.dec, pixel_format, &mut size)
        })?;

        buffer.resize(size, T::default());
        check_dec_status(unsafe {
            JxlDecoderSetImageOutBuffer(self.dec, pixel_format, buffer.as_mut_ptr().cast(), size)
        })?;

        Ok(())
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
    jpeg_reconstruction: bool,
    keep_orientation: bool,
    // TODO: Output preview if streaming. Currently always `false`
    need_preview: bool,
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
            jpeg_reconstruction: false,
            keep_orientation: false,
            need_preview: false,
        },
        memory_manager: None,
        parallel_runner: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThreadsRunner;
    use image::ImageError;

    #[test]
    fn test_decode() -> Result<(), ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut decoder = decoder_builder().build()?;

        let DecoderResult { info, data } = decoder.decode::<u8>(&sample)?;

        assert_eq!(data.len(), (info.width * info.height * 4) as _);

        // Check if icc profile is valid
        qcms::Profile::new_from_slice(&info.icc_profile).unwrap();

        Ok(())
    }

    #[test]
    #[ignore = "Upstream not implemented"]
    fn test_decode_container() -> Result<(), ImageError> {
        todo!()
    }

    #[test]
    fn test_decoder_builder() -> Result<(), ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let parallel_runner = ThreadsRunner::default();
        let mut decoder = decoder_builder()
            .num_channels(3)
            .endian(JxlEndianness::Big)
            .keep_orientation(true)
            .parallel_runner(&parallel_runner)
            .build()?;

        let DecoderResult { info, data, .. } = decoder.decode::<u8>(&sample)?;

        assert_eq!(data.len(), (info.width * info.height * 3) as _);

        Ok(())
    }

    #[test]
    fn test_decoder_streaming() -> Result<(), ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut decoder = decoder_builder().build()?;

        let DecoderResult { info, data } = decoder.decode::<u8>(&sample)?;

        assert_eq!(data.len(), (info.width * info.height * 3) as _);

        Ok(())
    }

    #[test]
    fn test_memory_manager() -> Result<(), Box<dyn std::error::Error>> {
        use crate::memory::MallocManager;

        let sample = std::fs::read("test/sample.jxl")?;
        let memory_manager = MallocManager::default();

        let mut decoder = decoder_builder().memory_manager(&memory_manager).build()?;
        let custom_buffer = decoder.decode::<u8>(&sample)?;

        decoder = decoder_builder().build()?;
        let default_buffer = decoder.decode::<u8>(&sample)?;

        assert!(
            custom_buffer.data == default_buffer.data,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
