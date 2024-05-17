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

//! Decoder of JPEG XL format

use std::{mem::MaybeUninit, ptr::null};

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::{
    codestream_header::{JxlBasicInfo, JxlOrientation},
    decode::*,
    types::{JxlDataType, JxlPixelFormat},
};

use crate::{
    common::{Endianness, PixelType},
    errors::{check_dec_status, DecodeError},
    memory::MemoryManager,
    parallel::JxlParallelRunner,
    utils::check_valid_signature,
};

mod result;
pub use result::*;

/// Basic information
pub type BasicInfo = JxlBasicInfo;
/// Progressive decoding steps
pub type ProgressiveDetail = JxlProgressiveDetail;
/// Orientation
pub type Orientation = JxlOrientation;

/// Desired Pixel Format
#[derive(Clone, Copy, Debug)]
pub struct PixelFormat {
    /// Amount of channels available in a pixel buffer.
    ///
    /// 1. single-channel data, e.g. grayscale or a single extra channel
    /// 2. single-channel + alpha
    /// 3. trichromatic, e.g. RGB
    /// 4. trichromatic + alpha
    ///
    /// # Default
    /// 0, which means determined automatically from color channels and alpha bits
    pub num_channels: u32,
    /// Whether multibyte data types are represented in big endian or little
    /// endian format. This applies to `u16`, `f16`, and `f32`.
    ///
    /// # Default
    /// [`Endianness::Native`]
    pub endianness: Endianness,
    /// Align scanlines to a multiple of align bytes.
    ///
    /// # Default
    /// 0, which means requiring no alignment (which has the same effect as value 1)
    pub align: usize,
}

impl Default for PixelFormat {
    fn default() -> Self {
        Self {
            num_channels: 0,
            endianness: Endianness::Native,
            align: 0,
        }
    }
}

/// JPEG XL Decoder
#[derive(Builder)]
#[builder(build_fn(skip, error = "None"))]
#[builder(setter(strip_option))]
pub struct JxlDecoder<'pr, 'mm> {
    /// Opaque pointer to the underlying decoder
    #[builder(setter(skip))]
    dec: *mut jpegxl_sys::decode::JxlDecoder,

    /// Override desired pixel format
    pub pixel_format: Option<PixelFormat>,

    /// Enables or disables preserving of as-in-bitstream pixel data orientation.
    /// If it is set to `true`, the decoder will skip applying the transformation
    ///
    /// # Default
    /// `false`, and the returned pixel data is re-oriented
    pub skip_reorientation: Option<bool>,
    /// Enables or disables preserving of associated alpha channels.
    /// If it is set to `true`, the colors will be unpremultiplied based on the alpha channel
    ///
    /// # Default
    /// `false`, and return the pixel data "as is".
    pub unpremul_alpha: Option<bool>,
    /// Enables or disables rendering spot colors.
    /// If it is set to `false`, then spot colors are not rendered, and have to be retrieved
    /// separately. This is useful for printing applications
    ///
    /// # Default
    /// `true`, and spot colors are rendered, which is OK for viewing the decoded image
    pub render_spotcolors: Option<bool>,
    /// Enables or disables coalescing of zero-duration frames.
    /// For loading a multi-layer still image as separate layers (as opposed to the merged image),
    /// coalescing has to be disabled
    ///
    /// # Default
    /// `true`, and all frames have the image dimensions, and are blended if needed.
    pub coalescing: Option<bool>,
    /// Perform tone mapping to the peak display luminance.
    ///
    /// # Note
    /// This is provided for convenience and the exact tone mapping that is performed
    /// is not meant to be considered authoritative in any way. It may change from version
    /// to version
    pub desired_intensity_target: Option<f32>,
    /// Configures whether to get boxes in raw mode or in decompressed mode.
    ///
    /// # Default
    /// false, and the boxes are returned in raw mode
    pub decompress: Option<bool>,

    /// Configures at which progressive steps in frame decoding
    ///
    /// # Default
    /// [`ProgressiveDetail::DC`]
    pub progressive_detail: Option<JxlProgressiveDetail>,

    /// Set if need ICC profile
    ///
    /// # Default
    /// `false`
    pub icc_profile: bool,

    /// Set initial buffer for JPEG reconstruction
    /// Larger buffer could make reconstruction faster by doing fewer reallocation
    ///
    /// Default: 512 KiB
    pub init_jpeg_buffer: usize,

    /// Set parallel runner
    pub parallel_runner: Option<&'pr dyn JxlParallelRunner>,

    /// Set memory manager
    pub memory_manager: Option<&'mm dyn MemoryManager>,
}

impl<'pr, 'mm> JxlDecoderBuilder<'pr, 'mm> {
    /// Build a [`JxlDecoder`]
    ///
    /// # Errors
    /// Return [`DecodeError::CannotCreateDecoder`] if it fails to create the decoder.
    pub fn build(&self) -> Result<JxlDecoder<'pr, 'mm>, DecodeError> {
        let mm = self.memory_manager.flatten();
        let dec = unsafe {
            mm.map_or_else(
                || JxlDecoderCreate(null()),
                |mm| JxlDecoderCreate(&mm.manager()),
            )
        };

        if dec.is_null() {
            return Err(DecodeError::CannotCreateDecoder);
        }

        Ok(JxlDecoder {
            dec,
            pixel_format: self.pixel_format.flatten(),
            skip_reorientation: self.skip_reorientation.flatten(),
            unpremul_alpha: self.unpremul_alpha.flatten(),
            render_spotcolors: self.render_spotcolors.flatten(),
            coalescing: self.coalescing.flatten(),
            desired_intensity_target: self.desired_intensity_target.flatten(),
            decompress: self.decompress.flatten(),
            progressive_detail: self.progressive_detail.flatten(),
            icc_profile: self.icc_profile.unwrap_or_default(),
            init_jpeg_buffer: self.init_jpeg_buffer.unwrap_or(512 * 1024),
            parallel_runner: self.parallel_runner.flatten(),
            memory_manager: mm,
        })
    }
}

impl<'pr, 'mm> JxlDecoder<'pr, 'mm> {
    pub(crate) fn decode_internal(
        &self,
        data: &[u8],
        data_type: Option<JxlDataType>,
        with_icc_profile: bool,
        mut reconstruct_jpeg_buffer: Option<&mut Vec<u8>>,
        format: *mut JxlPixelFormat,
        pixels: &mut Vec<u8>,
    ) -> Result<Metadata, DecodeError> {
        let Some(sig) = check_valid_signature(data) else {
            return Err(DecodeError::InvalidInput);
        };
        if !sig {
            return Err(DecodeError::InvalidInput);
        }

        let mut basic_info = MaybeUninit::uninit();
        let mut icc = if with_icc_profile { Some(vec![]) } else { None };

        self.setup_decoder(with_icc_profile, reconstruct_jpeg_buffer.is_some())?;

        let next_in = data.as_ptr();
        let avail_in = std::mem::size_of_val(data) as _;

        check_dec_status(unsafe { JxlDecoderSetInput(self.dec, next_in, avail_in) })?;
        unsafe { JxlDecoderCloseInput(self.dec) };

        let mut status;
        loop {
            use JxlDecoderStatus as s;

            status = unsafe { JxlDecoderProcessInput(self.dec) };

            match status {
                s::NeedMoreInput | s::Error => return Err(DecodeError::GenericError),

                // Get the basic info
                s::BasicInfo => {
                    check_dec_status(unsafe {
                        JxlDecoderGetBasicInfo(self.dec, basic_info.as_mut_ptr())
                    })?;

                    if let Some(pr) = self.parallel_runner {
                        pr.callback_basic_info(unsafe { &*basic_info.as_ptr() });
                    }
                }

                // Get color encoding
                s::ColorEncoding => {
                    self.get_icc_profile(unsafe { icc.as_mut().unwrap_unchecked() })?;
                }

                // Get JPEG reconstruction buffer
                s::JpegReconstruction => {
                    // Safety: JpegReconstruction is only called when reconstruct_jpeg_buffer
                    // is not None
                    let buf = unsafe { reconstruct_jpeg_buffer.as_mut().unwrap_unchecked() };
                    buf.resize(self.init_jpeg_buffer, 0);
                    check_dec_status(unsafe {
                        JxlDecoderSetJPEGBuffer(self.dec, buf.as_mut_ptr(), buf.len())
                    })?;
                }

                // JPEG buffer need more space
                s::JpegNeedMoreOutput => {
                    // Safety: JpegNeedMoreOutput is only called when reconstruct_jpeg_buffer
                    // is not None
                    let buf = unsafe { reconstruct_jpeg_buffer.as_mut().unwrap_unchecked() };
                    let need_to_write = unsafe { JxlDecoderReleaseJPEGBuffer(self.dec) };

                    buf.resize(buf.len() + need_to_write, 0);
                    check_dec_status(unsafe {
                        JxlDecoderSetJPEGBuffer(self.dec, buf.as_mut_ptr(), buf.len())
                    })?;
                }

                // Get the output buffer
                s::NeedImageOutBuffer => {
                    self.output(unsafe { &*basic_info.as_ptr() }, data_type, format, pixels)?;
                }

                s::FullImage => continue,
                s::Success => {
                    if let Some(buf) = reconstruct_jpeg_buffer.as_mut() {
                        let remaining = unsafe { JxlDecoderReleaseJPEGBuffer(self.dec) };

                        buf.truncate(buf.len() - remaining);
                        buf.shrink_to_fit();
                    }

                    unsafe { JxlDecoderReset(self.dec) };

                    let info = unsafe { basic_info.assume_init() };
                    return Ok(Metadata {
                        width: info.xsize,
                        height: info.ysize,
                        intensity_target: info.intensity_target,
                        min_nits: info.min_nits,
                        orientation: info.orientation,
                        num_color_channels: info.num_color_channels,
                        has_alpha_channel: info.alpha_bits > 0,
                        intrinsic_width: info.intrinsic_xsize,
                        intrinsic_height: info.intrinsic_ysize,
                        icc_profile: icc,
                    });
                }
                s::NeedPreviewOutBuffer => todo!(),
                s::BoxNeedMoreOutput => todo!(),
                s::PreviewImage => todo!(),
                s::Frame => todo!(),
                s::Box => todo!(),
                s::FrameProgression => todo!(),
            }
        }
    }

    fn setup_decoder(&self, icc: bool, reconstruct_jpeg: bool) -> Result<(), DecodeError> {
        if let Some(runner) = self.parallel_runner {
            check_dec_status(unsafe {
                JxlDecoderSetParallelRunner(self.dec, runner.runner(), runner.as_opaque_ptr())
            })?;
        }

        let events = {
            use JxlDecoderStatus::{BasicInfo, ColorEncoding, FullImage, JpegReconstruction};

            let mut events = BasicInfo as i32 | FullImage as i32;
            if icc {
                events |= ColorEncoding as i32;
            }
            if reconstruct_jpeg {
                events |= JpegReconstruction as i32;
            }

            events
        };
        check_dec_status(unsafe { JxlDecoderSubscribeEvents(self.dec, events) })?;

        if let Some(val) = self.skip_reorientation {
            check_dec_status(unsafe { JxlDecoderSetKeepOrientation(self.dec, val.into()) })?;
        }
        if let Some(val) = self.unpremul_alpha {
            check_dec_status(unsafe { JxlDecoderSetUnpremultiplyAlpha(self.dec, val.into()) })?;
        }
        if let Some(val) = self.render_spotcolors {
            check_dec_status(unsafe { JxlDecoderSetRenderSpotcolors(self.dec, val.into()) })?;
        }
        if let Some(val) = self.coalescing {
            check_dec_status(unsafe { JxlDecoderSetCoalescing(self.dec, val.into()) })?;
        }
        if let Some(val) = self.desired_intensity_target {
            check_dec_status(unsafe { JxlDecoderSetDesiredIntensityTarget(self.dec, val) })?;
        }

        Ok(())
    }

    fn get_icc_profile(&self, icc_profile: &mut Vec<u8>) -> Result<(), DecodeError> {
        let mut icc_size = 0;
        check_dec_status(unsafe {
            JxlDecoderGetICCProfileSize(self.dec, JxlColorProfileTarget::Data, &mut icc_size)
        })?;
        icc_profile.resize(icc_size, 0);

        check_dec_status(unsafe {
            JxlDecoderGetColorAsICCProfile(
                self.dec,
                JxlColorProfileTarget::Data,
                icc_profile.as_mut_ptr(),
                icc_size,
            )
        })?;

        Ok(())
    }

    fn output(
        &self,
        info: &BasicInfo,
        data_type: Option<JxlDataType>,
        format: *mut JxlPixelFormat,
        pixels: &mut Vec<u8>,
    ) -> Result<(), DecodeError> {
        let data_type = match data_type {
            Some(v) => v,
            None => match (info.bits_per_sample, info.exponent_bits_per_sample) {
                (x, 0) if x <= 8 => JxlDataType::Uint8,
                (x, 0) if x <= 16 => JxlDataType::Uint16,
                (16, _) => JxlDataType::Float16,
                (32, _) => JxlDataType::Float,
                (x, _) => return Err(DecodeError::UnsupportedBitWidth(x)),
            },
        };

        let f = self.pixel_format.unwrap_or_default();
        let pixel_format = JxlPixelFormat {
            num_channels: if f.num_channels == 0 {
                info.num_color_channels + u32::from(info.alpha_bits > 0)
            } else {
                f.num_channels
            },
            data_type,
            endianness: f.endianness,
            align: f.align,
        };

        let mut size = 0;
        check_dec_status(unsafe {
            JxlDecoderImageOutBufferSize(self.dec, &pixel_format, &mut size)
        })?;
        pixels.resize(size, 0);

        check_dec_status(unsafe {
            JxlDecoderSetImageOutBuffer(self.dec, &pixel_format, pixels.as_mut_ptr().cast(), size)
        })?;

        unsafe { *format = pixel_format };
        Ok(())
    }

    /// Decode a JPEG XL image
    ///
    /// # Errors
    /// Return a [`DecodeError`] when internal decoder fails
    pub fn decode(&self, data: &[u8]) -> Result<(Metadata, Pixels), DecodeError> {
        let mut buffer = vec![];
        let mut pixel_format = MaybeUninit::uninit();
        let metadata = self.decode_internal(
            data,
            None,
            self.icc_profile,
            None,
            pixel_format.as_mut_ptr(),
            &mut buffer,
        )?;
        Ok((
            metadata,
            Pixels::new(buffer, unsafe { &pixel_format.assume_init() }),
        ))
    }

    /// Decode a JPEG XL image to a specific pixel type
    ///
    /// # Errors
    /// Return a [`DecodeError`] when internal decoder fails
    pub fn decode_with<T: PixelType>(
        &self,
        data: &[u8],
    ) -> Result<(Metadata, Vec<T>), DecodeError> {
        let mut buffer = vec![];
        let mut pixel_format = MaybeUninit::uninit();
        let metadata = self.decode_internal(
            data,
            Some(T::pixel_type()),
            self.icc_profile,
            None,
            pixel_format.as_mut_ptr(),
            &mut buffer,
        )?;

        // Safety: type `T` is set by user and provide to the decoder to determine output data type
        let buf = unsafe {
            let pixel_format = pixel_format.assume_init();
            debug_assert!(T::pixel_type() == pixel_format.data_type);
            T::convert(&buffer, &pixel_format)
        };

        Ok((metadata, buf))
    }

    /// Reconstruct JPEG data. Fallback to pixels if JPEG reconstruction fails
    ///
    /// # Note
    /// You can reconstruct JPEG data or get pixels in one go
    ///
    /// # Errors
    /// Return a [`DecodeError`] when internal decoder fails
    pub fn reconstruct(&self, data: &[u8]) -> Result<(Metadata, Data), DecodeError> {
        let mut buffer = vec![];
        let mut pixel_format = MaybeUninit::uninit();
        let mut jpeg_buf = vec![];
        let metadata = self.decode_internal(
            data,
            None,
            self.icc_profile,
            Some(&mut jpeg_buf),
            pixel_format.as_mut_ptr(),
            &mut buffer,
        )?;

        Ok((
            metadata,
            if jpeg_buf.is_empty() {
                Data::Pixels(Pixels::new(buffer, unsafe { &pixel_format.assume_init() }))
            } else {
                Data::Jpeg(jpeg_buf)
            },
        ))
    }
}

impl<'prl, 'mm> Drop for JxlDecoder<'prl, 'mm> {
    fn drop(&mut self) {
        unsafe { JxlDecoderDestroy(self.dec) };
    }
}

/// Return a [`JxlDecoderBuilder`] with default settings
#[must_use]
pub fn decoder_builder<'prl, 'mm>() -> JxlDecoderBuilder<'prl, 'mm> {
    JxlDecoderBuilder::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn test_derive() {
        let e = PixelFormat::default().clone();
        println!("{e:?}");

        _ = decoder_builder().clone();
    }
}
