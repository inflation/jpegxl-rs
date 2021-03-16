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
};

/// Basic Information
pub type BasicInfo = JxlBasicInfo;

/// Result of decoding
pub struct JxlDecoderResult<T: PixelType> {
    /// Basic information
    pub info: BasicInfo,
    /// ICC color profile
    pub icc_profile: Vec<u8>,
    /// Decoded image data
    pub data: Vec<T>,
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

    /// Decode a JPEG XL image.<br />
    /// Currently only support RGB(A)8/16/32 encoded static image. Transformation info are discarded.
    /// # Errors
    /// Return a [`DecodeError`] when internal decoder fails
    pub fn decode<T: PixelType>(
        &mut self,
        data: &[u8],
    ) -> Result<JxlDecoderResult<T>, DecodeError> {
        unsafe {
            let signature = JxlSignatureCheck(data.as_ptr(), data.len());
            if signature != JxlSignature::Codestream && signature != JxlSignature::Container {
                return Err(DecodeError::InvalidFileFormat);
            }

            if let Some(runner) = self.parallel_runner {
                check_dec_status(JxlDecoderSetParallelRunner(
                    self.dec,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                ))?
            }

            let events = jxl_dec_events!(
                JxlDecoderStatus::BasicInfo,
                JxlDecoderStatus::ColorEncoding,
                if self.options.jpeg_reconstruction {
                    JxlDecoderStatus::JpegReconstruction
                } else {
                    JxlDecoderStatus::FullImage
                }
            );

            // Stop after getting the basic info and decoding the image
            check_dec_status(JxlDecoderSubscribeEvents(self.dec, events))?;

            let next_in = data.as_ptr();
            let avail_in = std::mem::size_of_val(data) as _;

            let mut basic_info = None;
            let mut pixel_format = None;
            let mut icc_profile = Vec::new();
            let mut buffer: Vec<T> = Vec::new();

            check_dec_status(JxlDecoderSetInput(self.dec, next_in, avail_in))?;

            let mut status;
            loop {
                status = JxlDecoderProcessInput(self.dec);

                match status {
                    JxlDecoderStatus::Error => return Err(DecodeError::GenericError),
                    JxlDecoderStatus::NeedMoreInput => return Err(DecodeError::NeedMoreInput),

                    // Get the basic info
                    JxlDecoderStatus::BasicInfo => {
                        self.get_basic_info::<T>(&mut basic_info, &mut pixel_format)?;
                    }

                    // Get color encoding
                    JxlDecoderStatus::ColorEncoding => {
                        if let Some(format) = &pixel_format {
                            self.get_icc_profile(format, &mut icc_profile)?;
                        } else {
                            return Err(DecodeError::GenericError);
                        }
                    }

                    // Get JPEG reconstruction buffer
                    // TODO: Upstream not implemented
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
                    JxlDecoderStatus::NeedImageOutBuffer => {
                        let mut size = 0;
                        if let Some(format) = &pixel_format {
                            check_dec_status(JxlDecoderImageOutBufferSize(
                                self.dec, format, &mut size,
                            ))?;

                            buffer = Vec::with_capacity(size);
                            buffer.set_len(size);
                            check_dec_status(JxlDecoderSetImageOutBuffer(
                                self.dec,
                                format,
                                buffer.as_mut_ptr().cast(),
                                size,
                            ))?;
                        } else {
                            return Err(DecodeError::GenericError);
                        };
                    }

                    JxlDecoderStatus::FullImage => continue,
                    JxlDecoderStatus::Success => {
                        JxlDecoderReset(self.dec);
                        return basic_info.map_or(Err(DecodeError::GenericError), |info| {
                            Ok(JxlDecoderResult {
                                info,
                                icc_profile,
                                data: buffer,
                            })
                        });
                    }
                    _ => return Err(DecodeError::UnknownStatus(status)),
                }
            }
        }
    }

    unsafe fn get_basic_info<T: PixelType>(
        &mut self,
        basic_info: &mut Option<JxlBasicInfo>,
        pixel_format: &mut Option<JxlPixelFormat>,
    ) -> Result<(), DecodeError> {
        let mut info = JxlBasicInfo::new_uninit();
        check_dec_status(JxlDecoderGetBasicInfo(self.dec, info.as_mut_ptr()))?;
        let info = info.assume_init();
        *basic_info = Some(info);

        let mut format = JxlPixelFormat::new_uninit();
        check_dec_status(JxlDecoderDefaultPixelFormat(self.dec, format.as_mut_ptr()))?;
        let format = format.assume_init();

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

    unsafe fn get_icc_profile(
        &mut self,
        format: &JxlPixelFormat,
        icc_profile: &mut Vec<u8>,
    ) -> Result<(), DecodeError> {
        let mut icc_size = 0;

        check_dec_status(JxlDecoderGetICCProfileSize(
            self.dec,
            format,
            JxlColorProfileTarget::Data,
            &mut icc_size,
        ))?;

        icc_profile.resize(icc_size, 0);

        check_dec_status(JxlDecoderGetColorAsICCProfile(
            self.dec,
            format,
            JxlColorProfileTarget::Data,
            icc_profile.as_mut_ptr(),
            icc_size,
        ))?;

        Ok(())
    }
}

impl<'a> Drop for JxlDecoder<'a> {
    fn drop(&mut self) {
        unsafe { JxlDecoderDestroy(self.dec) };
    }
}

/// Builder for [`JxlDecoder`]
pub struct JxlDecoderBuilder<'a> {
    options: DecoderOptions,
    memory_manager: Option<&'a dyn JxlMemoryManager>,
    parallel_runner: Option<&'a dyn JxlParallelRunner>,
}

#[derive(Clone)]
struct DecoderOptions {
    num_channels: Option<u32>,
    endianness: Option<JxlEndianness>,
    align: Option<usize>,
    jpeg_reconstruction: bool,
}

impl<'a> JxlDecoderBuilder<'a> {
    /// Set number of channels for returned result
    #[must_use]
    pub fn num_channels(&mut self, num: u32) -> &mut Self {
        self.options.num_channels = Some(num);
        self
    }

    /// Set endianness for returned result
    #[must_use]
    pub fn endian(&mut self, endian: JxlEndianness) -> &mut Self {
        self.options.endianness = Some(endian);
        self
    }

    /// Set align for returned result
    #[must_use]
    pub fn align(&mut self, align: usize) -> &mut Self {
        self.options.align = Some(align);
        self
    }

    /// Set memory manager
    #[must_use]
    pub fn memory_manager(&mut self, memory_manager: &'a dyn JxlMemoryManager) -> &mut Self {
        self.memory_manager = Some(memory_manager);
        self
    }

    /// Set parallel runner
    #[must_use]
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
        },
        memory_manager: None,
        parallel_runner: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThreadsRunner;
    use ::image::ImageError;

    #[test]
    fn test_decode() -> Result<(), ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut decoder = decoder_builder().build()?;

        let JxlDecoderResult {
            info,
            data,
            icc_profile,
        } = decoder.decode::<u8>(&sample)?;

        assert_eq!(data.len(), (info.xsize * info.ysize * 4) as usize);

        // Check if icc profile is valid
        qcms::Profile::new_from_slice(&icc_profile).unwrap();

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
            .parallel_runner(&parallel_runner)
            .build()?;

        let JxlDecoderResult { info, data, .. } = decoder.decode::<u8>(&sample)?;

        assert_eq!(data.len(), (info.xsize * info.ysize * 3) as usize);

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
