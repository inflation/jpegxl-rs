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

//! Encoder of JPEG XL format

use std::ptr::null;

use bon::bon;
#[allow(clippy::wildcard_imports)]
use jpegxl_sys::encoder::encode::*;

use eros::{type_set::Contains, ErrorUnion, ReshapeUnion, TypeSet};

use crate::{
    common::PixelType,
    errors::{ruled_out, InternalError, InvalidState},
    memory::MemoryManager,
    parallel::ParallelRunner,
};

mod error;
pub use error::*;
mod options;
pub use options::*;

mod metadata;
pub use metadata::*;

mod frame;
pub use frame::*;

mod info;
pub use info::*;

mod session;
pub use session::*;

// MARK: Encoder

/// JPEG XL Encoder
#[allow(clippy::struct_excessive_bools)]
pub struct JxlEncoder<'prl, 'mm> {
    /// Opaque pointer to the underlying encoder
    enc: *mut jpegxl_sys::encoder::encode::JxlEncoder,

    /// Set alpha channel for [`Self::encode`] and [`Self::encode_frame`].
    /// A [`Session`] uses [`ImageInfo`] instead
    ///
    /// Default: false
    pub has_alpha: bool,
    /// Set lossless
    ///
    /// Default: false
    pub lossless: Option<bool>,
    /// Set speed
    ///
    /// Default: `Squirrel` (7).
    pub speed: EncoderSpeed,
    /// Set quality for lossy compression: target max butteraugli distance, lower = higher quality
    ///
    ///  Range: 0 .. 15.<br />
    ///    0.0 = mathematically lossless (however, use `lossless` to use true lossless). <br />
    ///    1.0 = visually lossless. <br />
    ///    Recommended range: 0.5 .. 3.0. <br />
    ///    Default value: 1.0. <br />
    ///    If `lossless` is set to `true`, this value is unused and implied to be 0.
    pub quality: f32,
    /// Configure the encoder to use the JPEG XL container format
    ///
    /// Using the JPEG XL container format allows one to store metadata such as JPEG reconstruction;
    /// but it adds a few bytes to the encoded file for container headers
    /// even if there is no extra metadata.
    pub use_container: bool,
    /// Configure the encoder to use the original color profile
    ///
    /// If the input image has a color profile, it will be used for the encoded image.
    /// Otherwise, an internal fixed color profile is chosen (which should be smaller).
    ///
    /// When lossless re-compressing JPEG image, you must set this to true.
    ///
    /// Default: `false`
    pub uses_original_profile: bool,
    /// Set the decoding speed tier
    ///
    /// Minimum is 0 (highest quality), and maximum is 4 (lowest quality). Default is 0.
    pub decoding_speed: i64,
    /// Set initial output buffer size in bytes.
    /// Anything less than 32 bytes will be rounded up to 32 bytes.
    ///
    /// Default: 512 KiB
    pub init_buffer_size: usize,

    /// Set color encoding
    ///
    /// Default: sRGB for int, Linear sRGB for float
    pub color_encoding: Option<ColorEncoding>,

    /// Set HDR target intensity.
    /// Specify the target intensity in nits for 1.0 value
    pub target_intensity: Option<f32>,

    /// Set parallel runner
    ///
    /// Default: `None`, indicating single thread execution
    pub parallel_runner: Option<&'prl dyn ParallelRunner>,

    /// Whether box is used in encoder
    use_box: bool,
    /// Raw frame options set by [`Self::set_frame_option`]
    options: Vec<(JxlEncoderFrameSettingId, i64)>,
    /// Boxes added to the next finished image
    boxes: Vec<([u8; 4], Vec<u8>, bool)>,

    /// Set memory manager
    #[allow(dead_code)]
    memory_manager: Option<&'mm dyn MemoryManager>,
}

#[bon]
impl<'prl, 'mm> JxlEncoder<'prl, 'mm> {
    /// Build a [`JxlEncoder`]
    ///
    /// # Errors
    /// Return [`CannotCreateEncoder`] if it fails to create the encoder
    #[builder(derive(Clone))]
    pub fn new(
        memory_manager: Option<&'mm dyn MemoryManager>,
        #[builder(default)] has_alpha: bool,
        lossless: Option<bool>,
        #[builder(default)] speed: EncoderSpeed,
        #[builder(default = 1.0)] quality: f32,
        #[builder(default)] use_container: bool,
        #[builder(default)] uses_original_profile: bool,
        #[builder(default)] decoding_speed: i64,
        init_buffer_size: Option<usize>,
        color_encoding: Option<ColorEncoding>,
        target_intensity: Option<f32>,
        parallel_runner: Option<&'prl dyn ParallelRunner>,
        #[builder(default)] use_box: bool,
    ) -> eros::Result<Self, (CannotCreateEncoder,)> {
        // SAFETY: libjxl copies the memory manager, so the temporary only has to outlive the call
        let enc = unsafe {
            memory_manager.map_or_else(
                || JxlEncoderCreate(null()),
                |mm| JxlEncoderCreate(&mm.manager()),
            )
        };

        if enc.is_null() {
            return Err(CannotCreateEncoder.into());
        }

        Ok(Self {
            enc,
            has_alpha,
            lossless,
            speed,
            quality,
            use_container,
            uses_original_profile,
            decoding_speed,
            init_buffer_size: init_buffer_size.map_or(512 * 1024, |v| if v < 32 { 32 } else { v }),
            color_encoding,
            target_intensity,
            parallel_runner,
            use_box,
            options: Vec::new(),
            boxes: Vec::new(),
            memory_manager,
        })
    }
}

use jxl_encoder_builder::{IsUnset, SetQuality, State};

impl<'prl, 'mm, S: State> JxlEncoderBuilder<'prl, 'mm, S> {
    /// Set the `quality` parameter from a JPEG-style quality factor (0-100, higher is better
    /// quality).
    #[allow(dead_code)]
    pub fn jpeg_quality(self, quality: f32) -> JxlEncoderBuilder<'prl, 'mm, SetQuality<S>>
    where
        S::Quality: IsUnset,
    {
        // SAFETY: this is a pure function of its argument
        self.quality(unsafe { JxlEncoderDistanceFromQuality(quality) })
    }
}

// MARK: Private helper functions
impl JxlEncoder<'_, '_> {
    /// Error mapping from underlying C const to [`EncoderFailure`], in any union that
    /// contains it
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn check_enc_status<S, I>(&self, status: JxlEncoderStatus) -> eros::Result<(), S>
    where
        S: TypeSet,
        S::Variants: Contains<EncoderFailure, I>,
    {
        let failure = match status {
            JxlEncoderStatus::Success => return Ok(()),
            // SAFETY: `self.enc` is valid until drop
            JxlEncoderStatus::Error => unsafe { JxlEncoderGetError(self.enc) }.into(),
            // Only `JxlEncoderProcessOutput` returns it, and `process_output` handles it
            JxlEncoderStatus::NeedMoreOutput => EncoderFailure::Unspecified,
        };
        Err(ErrorUnion::new(failure))
    }

    fn image_info<T: PixelType>(&self, width: u32, height: u32) -> ImageInfo {
        let (bits, exponent_bits) = T::bits_per_sample();
        ImageInfo::builder()
            .width(width)
            .height(height)
            .bits_per_sample(bits)
            .exponent_bits_per_sample(exponent_bits)
            .has_alpha(self.has_alpha)
            .build()
    }
}

// MARK: Public interface
impl<'prl, 'mm> JxlEncoder<'prl, 'mm> {
    /// Start an encoding session for an image.
    ///
    /// The encoder is reset when the session is dropped.
    ///
    /// # Errors
    /// Return [`EncoderFailure`] if the encoder cannot be configured
    pub fn session(
        &mut self,
        info: &ImageInfo,
    ) -> eros::Result<Session<'_, 'prl, 'mm>, (EncoderFailure,)> {
        Session::new(self, Some(info))
    }

    /// Start an encoding session whose image information comes from the first JPEG frame
    ///
    /// # Errors
    /// Return [`EncoderFailure`] if the encoder cannot be configured
    pub fn jpeg_session(&mut self) -> eros::Result<Session<'_, 'prl, 'mm>, (EncoderFailure,)> {
        Session::new(self, None)
    }

    /// The settings a frame uses unless it has its own
    #[must_use]
    pub fn frame_settings(&self) -> FrameSettings {
        FrameSettings {
            lossless: self.lossless,
            speed: self.speed,
            quality: self.quality,
            decoding_speed: self.decoding_speed,
            options: self.options.clone(),
        }
    }

    /// Set a specific encoder frame setting for every following frame.
    /// It overrides the fields of the encoder, e.g. [`JxlEncoderFrameSettingId::Effort`]
    /// overrides [`Self::speed`]. An invalid value is reported when a frame is added.
    pub fn set_frame_option(&mut self, option: JxlEncoderFrameSettingId, value: i64) {
        match self.options.iter_mut().find(|(id, _)| *id == option) {
            Some(entry) => entry.1 = value,
            None => self.options.push((option, value)),
        }
    }

    /// Add a metadata box to the next image. It is kept until an image is finished
    pub fn add_metadata(&mut self, metadata: &Metadata, compress: bool) {
        let (t, data) = metadata.parts();
        self.boxes.push((t, data.to_vec(), compress));
    }

    /// Encode a JPEG XL image from existing raw JPEG data
    ///
    /// Note: Ignore alpha channel settings
    ///
    /// # Errors
    /// Return [`EncoderFailure`] if the internal encoder fails to encode
    pub fn encode_jpeg(
        &mut self,
        data: &[u8],
    ) -> eros::Result<Vec<u8>, (EncoderFailure, InternalError)> {
        let mut session = self.jpeg_session().widen()?;
        session.store_jpeg_metadata().widen()?;
        session.add_jpeg_frame(data).widen()?;
        session
            .finish()
            .try_recover(ruled_out::<(InvalidState,), _, _, _>)
    }

    /// Encode a JPEG XL image from pixels, with the bit depth of `T`
    ///
    /// Note: Use RGB(3) channels, native endianness and no alignment.
    /// Ignore alpha channel settings
    ///
    /// # Errors
    /// Return [`EncoderFailure`] if the internal encoder fails to encode, or
    /// [`InvalidFrameName`]
    pub fn encode<T: PixelType>(
        &mut self,
        data: &[T],
        width: u32,
        height: u32,
    ) -> eros::Result<Vec<u8>, (EncoderFailure, InvalidFrameName, InternalError)> {
        self.encode_frame(&EncoderFrame::new(data), width, height)
    }

    /// Encode a JPEG XL image from a frame, with the bit depth of `T`.
    /// See [`EncoderFrame`] for custom options of the original pixels.
    ///
    /// # Errors
    /// Return [`EncoderFailure`] if the internal encoder fails to encode, or
    /// [`InvalidFrameName`]
    pub fn encode_frame<T: PixelType>(
        &mut self,
        frame: &EncoderFrame<T>,
        width: u32,
        height: u32,
    ) -> eros::Result<Vec<u8>, (EncoderFailure, InvalidFrameName, InternalError)> {
        let info = self.image_info::<T>(width, height);
        let mut session = self.session(&info).widen()?;
        session.add_frame(frame).widen()?;
        session
            .finish()
            .try_recover(ruled_out::<(InvalidState,), _, _, _>)
    }
}

impl Drop for JxlEncoder<'_, '_> {
    fn drop(&mut self) {
        // SAFETY: `self.enc` is valid and never used again
        unsafe { JxlEncoderDestroy(self.enc) };
    }
}

// SAFETY: JxlEncoder can be safely sent between threads. The underlying libjxl
// encoder does not store references to thread-local state. While libjxl uses a
// thread-local LCMS context for color management (see lib/jxl/cms/jxl_cms.cc),
// this context is looked up dynamically via GetContext() on each use, not stored
// in the encoder. Moving an encoder to another thread will use that thread's
// LCMS context for subsequent operations.
//
// Note: JxlEncoder is NOT Sync because the underlying C API is not safe for
// concurrent access from multiple threads.
unsafe impl Send for JxlEncoder<'_, '_> {}

/// Return a [`JxlEncoderBuilder`] with default settings
pub fn encoder_builder<'prl, 'mm>() -> JxlEncoderBuilder<'prl, 'mm> {
    JxlEncoder::builder()
}

// MARK: Tests
#[cfg(test)]
mod tests {
    use super::*;
    use testresult::TestResult;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_jpeg_quality() -> TestResult {
        let encoder = encoder_builder().jpeg_quality(100.0).build()?;
        assert_eq!(encoder.quality, 0.0);
        let encoder = encoder_builder().jpeg_quality(90.0).build()?;
        assert_eq!(encoder.quality, 1.0);
        Ok(())
    }

    #[test]
    fn test_metadata_queued() -> TestResult {
        let mut encoder = encoder_builder().build()?;
        encoder.add_metadata(&Metadata::Exif(&[0, 1, 2, 3]), true);
        assert_eq!(encoder.boxes.len(), 1);
        Ok(())
    }
}
