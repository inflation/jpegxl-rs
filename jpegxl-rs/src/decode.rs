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

use std::ptr::null;

use bon::bon;
use eros::{Context, ErrorUnion, IntoUnion, ReshapeUnion};
#[allow(clippy::wildcard_imports)]
use jpegxl_sys::{
    common::types::JxlDataType,
    decode::*,
    metadata::codestream_header::{JxlBasicInfo, JxlOrientation},
};

use crate::{
    common::{Endianness, PixelType},
    errors::{bug, InternalError, InvalidState},
    memory::MemoryManager,
    parallel::ParallelRunner,
    utils::check_valid_signature,
};

mod error;
pub use error::*;
mod event;
pub use event::*;
mod result;
pub use result::*;
mod session;
pub use session::*;

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
pub struct JxlDecoder<'pr, 'mm> {
    /// Opaque pointer to the underlying decoder
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
    pub parallel_runner: Option<&'pr dyn ParallelRunner>,

    /// Set memory manager
    pub memory_manager: Option<&'mm dyn MemoryManager>,
}

#[bon]
impl<'pr, 'mm> JxlDecoder<'pr, 'mm> {
    /// Build a [`JxlDecoder`]
    ///
    /// # Errors
    /// Return [`CannotCreateDecoder`] if it fails to create the decoder.
    #[builder(derive(Clone))]
    pub fn new(
        pixel_format: Option<PixelFormat>,
        skip_reorientation: Option<bool>,
        unpremul_alpha: Option<bool>,
        render_spotcolors: Option<bool>,
        coalescing: Option<bool>,
        desired_intensity_target: Option<f32>,
        decompress: Option<bool>,
        progressive_detail: Option<JxlProgressiveDetail>,
        #[builder(default)] icc_profile: bool,
        #[builder(default = 512 * 1024)] init_jpeg_buffer: usize,
        parallel_runner: Option<&'pr dyn ParallelRunner>,
        memory_manager: Option<&'mm dyn MemoryManager>,
    ) -> eros::Result<Self, (CannotCreateDecoder,)> {
        // SAFETY: libjxl copies the memory manager, so the temporary only has to outlive the call
        let dec = unsafe {
            memory_manager.map_or_else(
                || JxlDecoderCreate(null()),
                |mm| JxlDecoderCreate(&mm.manager()),
            )
        };

        if dec.is_null() {
            return Err(CannotCreateDecoder.into());
        }

        Ok(Self {
            dec,
            pixel_format,
            skip_reorientation,
            unpremul_alpha,
            render_spotcolors,
            coalescing,
            desired_intensity_target,
            decompress,
            progressive_detail,
            icc_profile,
            init_jpeg_buffer,
            parallel_runner,
            memory_manager,
        })
    }
}

/// Metadata, image of the last frame, and reconstructed JPEG of a one-shot decode
pub(crate) type Decoded = (Metadata, Option<Image>, Option<Vec<u8>>);

impl<'pr, 'mm> JxlDecoder<'pr, 'mm> {
    /// Start a decoding session that the caller drives event by event.
    ///
    /// The decoder is reset when the session is dropped.
    ///
    /// # Errors
    /// Return a [`GenericError`] if the decoder cannot be configured
    pub fn session(
        &mut self,
        events: Events,
    ) -> eros::Result<Session<'_, 'pr, 'mm>, (GenericError,)> {
        Session::new(self, events)
    }
}

impl JxlDecoder<'_, '_> {
    /// Run a [`Session`] over `data`. Returns the metadata, the image of the last frame
    /// and the reconstructed JPEG, if requested and possible.
    pub(crate) fn decode_internal(
        &self,
        data: &[u8],
        data_type: Option<JxlDataType>,
        with_icc_profile: bool,
        reconstruct_jpeg: bool,
    ) -> eros::Result<Decoded, DecodeErrors> {
        if !check_valid_signature(data).unwrap_or(false) {
            return Err(InvalidInput).union();
        }

        let mut events = Events::FULL_IMAGE;
        if with_icc_profile {
            events |= Events::COLOR_ENCODING;
        }
        if reconstruct_jpeg {
            events |= Events::JPEG_RECONSTRUCTION;
        }

        let mut session = Session::new(self, events)
            .context("configure the decoder")
            .widen()?;
        let (mut icc, mut image, mut jpeg) = (None, None, None);
        // Events come in order, so the information is always there
        let no_icc = |_: ErrorUnion<(NotAvailableYet,)>| bug("no ICC profile on its event");
        let no_info = |_: ErrorUnion<(InvalidState, NotAvailableYet)>| bug("no basic info");
        let mut input = data;
        loop {
            match session.process(&mut input).widen()? {
                Event::NeedMoreInput => return Err(IncompleteInput).union(),
                Event::ColorEncoding => {
                    icc = Some(
                        session
                            .icc_profile(ColorProfileTarget::Data)
                            .context("read the ICC profile")
                            .try_recover(no_icc)?,
                    );
                }
                Event::NeedImageOutBuffer => {
                    session
                        .alloc_image_buffer(self.pixel_format.unwrap_or_default(), data_type)
                        .context("allocate the image buffer")
                        .try_recover(no_info)?;
                }
                Event::FullImage(img) => image = img,
                Event::Jpeg(buf) => jpeg = Some(buf),
                Event::Success => break,
                _ => {}
            }
        }

        let info = session
            .basic_info()
            .ok_or(InternalError("No basic info"))
            .union()?;
        let metadata = Metadata {
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
        };
        Ok((metadata, image, jpeg))
    }

    /// Apply the options to the decoder. Called at the start of every session
    pub(crate) fn setup_decoder(&self, events: Events) -> eros::Result<(), (GenericError,)> {
        if let Some(runner) = self.parallel_runner {
            // SAFETY: `self.dec` is valid until drop and the runner outlives it
            check_dec_status(unsafe {
                JxlDecoderSetParallelRunner(self.dec, runner.runner(), runner.as_opaque_ptr())
            })?;
        }

        // SAFETY: `self.dec` is valid until drop
        check_dec_status(unsafe { JxlDecoderSubscribeEvents(self.dec, events.bits()) })?;

        if let Some(val) = self.skip_reorientation {
            // SAFETY: `self.dec` is valid until drop
            check_dec_status(unsafe { JxlDecoderSetKeepOrientation(self.dec, val.into()) })?;
        }
        if let Some(val) = self.unpremul_alpha {
            // SAFETY: `self.dec` is valid until drop
            check_dec_status(unsafe { JxlDecoderSetUnpremultiplyAlpha(self.dec, val.into()) })?;
        }
        if let Some(val) = self.render_spotcolors {
            // SAFETY: `self.dec` is valid until drop
            check_dec_status(unsafe { JxlDecoderSetRenderSpotcolors(self.dec, val.into()) })?;
        }
        if let Some(val) = self.coalescing {
            // SAFETY: `self.dec` is valid until drop
            check_dec_status(unsafe { JxlDecoderSetCoalescing(self.dec, val.into()) })?;
        }
        if let Some(val) = self.desired_intensity_target {
            // SAFETY: `self.dec` is valid until drop
            check_dec_status(unsafe { JxlDecoderSetDesiredIntensityTarget(self.dec, val) })?;
        }
        if let Some(val) = self.decompress {
            // SAFETY: `self.dec` is valid until drop
            check_dec_status(unsafe { JxlDecoderSetDecompressBoxes(self.dec, val.into()) })?;
        }
        if let Some(val) = self.progressive_detail {
            // SAFETY: `self.dec` is valid until drop
            check_dec_status(unsafe { JxlDecoderSetProgressiveDetail(self.dec, val) })?;
        }

        Ok(())
    }

    /// Decode a JPEG XL image
    ///
    /// # Errors
    /// Return one of [`DecodeErrors`] when decoding fails
    pub fn decode(&self, data: &[u8]) -> eros::Result<(Metadata, Pixels), DecodeErrors> {
        let (metadata, image, _) = self.decode_internal(data, None, self.icc_profile, false)?;
        let image = image.ok_or(InternalError("No image decoded")).union()?;
        Ok((metadata, image.into_pixels()))
    }

    /// Decode a JPEG XL image to a specific pixel type
    ///
    /// # Errors
    /// Return one of [`DecodeErrors`] when decoding fails
    pub fn decode_with<T: PixelType>(
        &self,
        data: &[u8],
    ) -> eros::Result<(Metadata, Vec<T>), DecodeErrors> {
        let (metadata, image, _) =
            self.decode_internal(data, Some(T::pixel_type()), self.icc_profile, false)?;
        let image = image.ok_or(InternalError("No image decoded")).union()?;

        debug_assert_eq!(T::pixel_type(), image.format.data_type);
        Ok((metadata, T::convert(&image.data, &image.format)))
    }

    /// Reconstruct JPEG data. Fallback to pixels if JPEG reconstruction fails
    ///
    /// # Note
    /// You can reconstruct JPEG data or get pixels in one go
    ///
    /// # Errors
    /// Return one of [`DecodeErrors`] when decoding fails
    pub fn reconstruct(&self, data: &[u8]) -> eros::Result<(Metadata, Data), DecodeErrors> {
        let (metadata, image, jpeg) = self.decode_internal(data, None, self.icc_profile, true)?;
        let data = jpeg
            .map(Data::Jpeg)
            .or_else(|| image.map(|image| Data::Pixels(image.into_pixels())))
            .ok_or(InternalError("No image decoded"))
            .union()?;
        Ok((metadata, data))
    }
}

impl Drop for JxlDecoder<'_, '_> {
    fn drop(&mut self) {
        // SAFETY: `self.dec` is valid and never used again
        unsafe { JxlDecoderDestroy(self.dec) };
    }
}

// SAFETY: JxlDecoder can be safely sent between threads. The underlying libjxl
// decoder does not store references to thread-local state. While libjxl uses a
// thread-local LCMS context for color management (see lib/jxl/cms/jxl_cms.cc),
// this context is looked up dynamically via GetContext() on each use, not stored
// in the decoder. Moving a decoder to another thread will use that thread's
// LCMS context for subsequent operations.
//
// Note: JxlDecoder is NOT Sync because the underlying C API is not safe for
// concurrent access from multiple threads.
unsafe impl Send for JxlDecoder<'_, '_> {}

/// Return a [`JxlDecoderBuilder`] with default settings
pub fn decoder_builder<'prl, 'mm>() -> JxlDecoderBuilder<'prl, 'mm> {
    JxlDecoder::builder()
}
