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

use std::{marker::PhantomData, mem::MaybeUninit, ops::Deref, ptr::null};

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::{
    color_encoding::JxlColorEncoding,
    encode::*,
    types::{JxlEndianness, JxlPixelFormat},
};

use crate::{
    common::PixelType, errors::EncodeError, memory::MemoryManager, parallel::JxlParallelRunner,
};

/// Encoding speed
#[derive(Debug, Clone, Copy)]
pub enum EncoderSpeed {
    /// Fastest, 1
    Lightning = 1,
    /// 2
    Thunder = 2,
    /// 3
    Falcon = 3,
    /// 4
    Cheetah,
    /// 5
    Hare,
    /// 6
    Wombat,
    /// 7, default
    Squirrel,
    /// 8
    Kitten,
    /// Slowest, 9
    Tortoise,
}

impl std::default::Default for EncoderSpeed {
    fn default() -> Self {
        Self::Squirrel
    }
}

/// Encoding color profile
#[derive(Debug, Clone, Copy)]
pub enum ColorEncoding {
    /// SRGB, default for uint pixel types
    Srgb,
    /// Linear SRGB, default for float pixel types
    LinearSrgb,
    /// SRGB, images with only luma channel
    SrgbLuma,
    /// Linear SRGB with only luma channel
    LinearSrgbLuma,
}

impl From<ColorEncoding> for JxlColorEncoding {
    fn from(val: ColorEncoding) -> Self {
        use ColorEncoding::{LinearSrgb, LinearSrgbLuma, Srgb, SrgbLuma};

        let mut color_encoding = MaybeUninit::uninit();

        unsafe {
            match val {
                Srgb => JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), false),
                LinearSrgb => JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), false),
                SrgbLuma => JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), true),
                LinearSrgbLuma => {
                    JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), true);
                }
            }
            color_encoding.assume_init()
        }
    }
}

/// BoxTypes
#[derive(Debug, Clone, Copy)]
enum BoxTypes {
    /// "Exif", a box with EXIF metadata
    Exif,
    /// "xml ", a box with XML data, in particular XMP metadata
    Xmp,
    /// "jumb", a JUMBF superbox, which can contain boxes with different types of metadata inside
    Jumb,
}

/// Metadata
#[derive(Debug, Clone, Copy)]
pub struct Metadata<'data> {
    exif: Option<&'data [u8]>,
    xmp: Option<&'data [u8]>,
    jumb: Option<&'data [u8]>,
    compress: bool,
}

impl<'data> Metadata<'data> {
    /// Create a default empty metadata.
    pub fn new() -> Self {
        Self {
            exif: None,
            xmp: None,
            jumb: None,
            compress: false,
        }
    }

    /// Add exif data
    pub fn exif(mut self, data: &'data [u8]) -> Self {
        self.exif = Some(data);
        self
    }

    /// Add xmp data
    pub fn xmp(mut self, data: &'data [u8]) -> Self {
        self.xmp = Some(data);
        self
    }

    /// Add jumb data
    pub fn jumb(mut self, data: &'data [u8]) -> Self {
        self.jumb = Some(data);
        self
    }

    /// Set metadata compression
    pub fn compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }
}

/// A frame for the encoder, consisting of the pixels and its options
pub struct EncoderFrame<'data, T: PixelType> {
    data: &'data [T],
    num_channels: Option<u32>,
    endianness: Option<JxlEndianness>,
    align: Option<usize>,
}

impl<'data, T: PixelType> EncoderFrame<'data, T> {
    /// Create a default frame from the data.
    ///
    /// Use RGB(3) channels, native endianness and no alignment.
    pub fn new(data: &'data [T]) -> Self {
        Self {
            data,
            num_channels: None,
            endianness: None,
            align: None,
        }
    }

    /// Set the number of channels of the source.
    ///
    /// _Note_: If you want to use alpha channel, add here
    #[must_use]
    pub fn num_channels(mut self, value: u32) -> Self {
        self.num_channels = Some(value);
        self
    }

    /// Set the endianness of the source.
    #[must_use]
    pub fn endianness(mut self, value: JxlEndianness) -> Self {
        self.endianness = Some(value);
        self
    }

    /// Set the align of the source.
    /// Align scanlines to a multiple of align bytes, or 0 to require no alignment at all
    #[must_use]
    pub fn align(mut self, value: usize) -> Self {
        self.align = Some(value);
        self
    }

    fn pixel_format(&self) -> JxlPixelFormat {
        JxlPixelFormat {
            num_channels: self.num_channels.unwrap_or(3),
            data_type: T::pixel_type(),
            endianness: self.endianness.unwrap_or(JxlEndianness::Native),
            align: self.align.unwrap_or(0),
        }
    }
}

/// Encoder result
pub struct EncoderResult<U: PixelType> {
    /// Output binary data
    pub data: Vec<u8>,
    _pixel_type: PhantomData<U>,
}

impl<U: PixelType> Deref for EncoderResult<U> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data.as_ref()
    }
}

/// JPEG XL Encoder
#[derive(Builder)]
#[builder(build_fn(skip, error = "None"))]
#[builder(setter(strip_option))]
#[allow(clippy::struct_excessive_bools)]
pub struct JxlEncoder<'prl, 'mm> {
    /// Opaque pointer to the underlying encoder
    #[builder(setter(skip))]
    enc: *mut jpegxl_sys::encode::JxlEncoder,
    /// Opaque pointer to the encoder options
    #[builder(setter(skip))]
    options_ptr: *mut JxlEncoderFrameSettings,

    /// Set alpha channel
    ///
    /// Default: false
    pub has_alpha: bool,
    /// Set lossless
    ///
    /// Default: false
    pub lossless: bool,
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
    /// Using the JPEG XL container format allows to store metadata such as JPEG reconstruction;
    /// but it adds a few bytes to the encoded file for container headers
    /// even if there is no extra metadata.
    pub use_container: bool,
    /// Configure the encoder to use the original color profile
    ///
    /// If the input image has a color profile, it will be used for the encoded image.
    /// Otherwise, an internal fixed color profile is chosen (which should be smaller).
    ///
    /// When lossless recompressing JPEG image, you must set this to true.
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
    /// Default: SRGB
    pub color_encoding: ColorEncoding,

    /// Set parallel runner
    ///
    /// Default: `None`, indicating single thread execution
    pub parallel_runner: Option<&'prl dyn JxlParallelRunner>,

    /// Set memory manager
    #[allow(dead_code)]
    memory_manager: Option<&'mm dyn MemoryManager>,
}

impl<'prl, 'mm> JxlEncoderBuilder<'prl, 'mm> {
    /// Build a [`JxlEncoder`]
    ///
    /// # Errors
    /// Return [`EncodeError::CannotCreateEncoder`] if it fails to create the encoder
    pub fn build(&self) -> Result<JxlEncoder<'prl, 'mm>, EncodeError> {
        let mm = self.memory_manager.flatten();
        let enc = unsafe {
            mm.map_or_else(
                || JxlEncoderCreate(null()),
                |mm| JxlEncoderCreate(&mm.manager()),
            )
        };

        if enc.is_null() {
            return Err(EncodeError::CannotCreateEncoder);
        }

        let options_ptr = unsafe { JxlEncoderFrameSettingsCreate(enc, null()) };

        let init_buffer_size =
            self.init_buffer_size
                .map_or(512 * 1024 * 1024, |v| if v < 32 { 32 } else { v });

        Ok(JxlEncoder {
            enc,
            options_ptr,
            has_alpha: self.has_alpha.unwrap_or_default(),
            lossless: self.lossless.unwrap_or_default(),
            speed: self.speed.unwrap_or_default(),
            quality: self.quality.unwrap_or(1.0),
            use_container: self.use_container.unwrap_or_default(),
            uses_original_profile: self.uses_original_profile.unwrap_or_default(),
            decoding_speed: self.decoding_speed.unwrap_or_default(),
            init_buffer_size,
            color_encoding: self.color_encoding.unwrap_or(ColorEncoding::Srgb),
            parallel_runner: self.parallel_runner.flatten(),
            memory_manager: mm,
        })
    }
}

// Private helper functions
impl JxlEncoder<'_, '_> {
    /// Error mapping from underlying C const to [`EncodeError`] enum
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn check_enc_status(&self, status: JxlEncoderStatus) -> Result<(), EncodeError> {
        match status {
            JxlEncoderStatus::Success => Ok(()),
            JxlEncoderStatus::Error => match unsafe { JxlEncoderGetError(self.enc) } {
                JxlEncoderError::OK => unreachable!(),
                JxlEncoderError::Generic => Err(EncodeError::GenericError),
                JxlEncoderError::OutOfMemory => Err(EncodeError::OutOfMemory),
                JxlEncoderError::Jbrd => Err(EncodeError::Jbrd),
                JxlEncoderError::BadInput => Err(EncodeError::BadInput),
                JxlEncoderError::NotSupported => Err(EncodeError::NotSupported),
                JxlEncoderError::ApiUsage => Err(EncodeError::ApiUsage),
            },
            JxlEncoderStatus::NeedMoreOutput => Err(EncodeError::NeedMoreOutput),
        }
    }

    // Set options
    fn set_options(&self) -> Result<(), EncodeError> {
        self.check_enc_status(unsafe { JxlEncoderUseContainer(self.enc, self.use_container) })?;
        self.check_enc_status(unsafe {
            JxlEncoderSetFrameLossless(self.options_ptr, self.lossless)
        })?;
        self.check_enc_status(unsafe {
            JxlEncoderFrameSettingsSetOption(
                self.options_ptr,
                FrameSetting::Effort,
                self.speed as _,
            )
        })?;
        self.check_enc_status(unsafe {
            JxlEncoderSetFrameDistance(self.options_ptr, self.quality)
        })?;
        self.check_enc_status(unsafe {
            JxlEncoderFrameSettingsSetOption(
                self.options_ptr,
                FrameSetting::DecodingSpeed,
                self.decoding_speed,
            )
        })?;

        Ok(())
    }

    // Setup the encoder
    fn setup_encoder(
        &self,
        width: u32,
        height: u32,
        (bits, exp): (u32, u32),
        has_alpha: bool,
    ) -> Result<(), EncodeError> {
        if let Some(runner) = self.parallel_runner {
            unsafe {
                self.check_enc_status(JxlEncoderSetParallelRunner(
                    self.enc,
                    runner.runner(),
                    runner.as_opaque_ptr(),
                ))?;
            }
        }

        self.set_options()?;

        let mut basic_info = unsafe {
            let mut info = MaybeUninit::uninit();
            JxlEncoderInitBasicInfo(info.as_mut_ptr());
            info.assume_init()
        };

        basic_info.xsize = width;
        basic_info.ysize = height;
        basic_info.have_container = self.use_container.into();
        basic_info.uses_original_profile = self.uses_original_profile.into();

        basic_info.bits_per_sample = bits;
        basic_info.exponent_bits_per_sample = exp;

        if has_alpha {
            basic_info.num_extra_channels = 1;
            basic_info.alpha_bits = bits;
            basic_info.alpha_exponent_bits = exp;
        } else {
            basic_info.num_extra_channels = 0;
            basic_info.alpha_bits = 0;
            basic_info.alpha_exponent_bits = 0;
        }

        match self.color_encoding {
            ColorEncoding::SrgbLuma | ColorEncoding::LinearSrgbLuma => {
                basic_info.num_color_channels = 1;
            }
            _ => (),
        }

        if let Some(pr) = self.parallel_runner {
            pr.callback_basic_info(&basic_info);
        }

        self.check_enc_status(unsafe { JxlEncoderSetBasicInfo(self.enc, &basic_info) })?;

        self.check_enc_status(unsafe {
            JxlEncoderSetColorEncoding(self.enc, &self.color_encoding.into())
        })
    }

    // Add box
    fn add_box(
        &self,
        box_contents: &[u8],
        box_type: BoxTypes,
        compress: bool,
    ) -> Result<(), EncodeError> {
        let mut box_type = match box_type {
            BoxTypes::Exif => [69, 120, 105, 102],
            BoxTypes::Xmp => [120, 109, 108, 32],
            BoxTypes::Jumb => [106, 117, 109, 98],
        };
        self.check_enc_status(unsafe {
            JxlEncoderAddBox(
                self.enc,
                &mut box_type,
                box_contents.as_ptr().cast(),
                box_contents.len(),
                compress.into(),
            )
        })
    }

    // Add metadata
    fn add_metadata(&self, metadata: Metadata) -> Result<(), EncodeError> {
        self.check_enc_status(unsafe { JxlEncoderUseBoxes(self.enc) })?;
        let compress = metadata.compress;
        if let Some(data) = metadata.exif {
            self.add_box(data, BoxTypes::Exif, compress)?;
        };
        if let Some(data) = metadata.xmp {
            self.add_box(data, BoxTypes::Xmp, compress)?;
        };
        if let Some(data) = metadata.jumb {
            self.add_box(data, BoxTypes::Jumb, compress)?;
        };
        unsafe { JxlEncoderCloseBoxes(self.enc) };
        Ok(())
    }

    // Add a frame
    fn add_frame<T: PixelType>(&self, frame: &EncoderFrame<T>) -> Result<(), EncodeError> {
        self.check_enc_status(unsafe {
            JxlEncoderAddImageFrame(
                self.options_ptr,
                &frame.pixel_format(),
                frame.data.as_ptr().cast(),
                std::mem::size_of_val(frame.data),
            )
        })
    }

    // Add a frame from JPEG raw data
    fn add_jpeg_frame(&self, data: &[u8]) -> Result<(), EncodeError> {
        self.check_enc_status(unsafe {
            JxlEncoderAddJPEGFrame(
                self.options_ptr,
                data.as_ptr().cast(),
                std::mem::size_of_val(data),
            )
        })
    }

    fn _internal(&mut self) -> Result<Vec<u8>, EncodeError> {
        unsafe { JxlEncoderCloseInput(self.enc) };

        let mut buffer = vec![0; self.init_buffer_size];
        let mut next_out = buffer.as_mut_ptr().cast();
        let mut avail_out = buffer.len();

        let mut status;
        loop {
            status = unsafe { JxlEncoderProcessOutput(self.enc, &mut next_out, &mut avail_out) };

            if status != JxlEncoderStatus::NeedMoreOutput {
                break;
            }

            unsafe {
                let offset = next_out.offset_from(buffer.as_ptr());
                debug_assert!(offset >= 0);

                buffer.resize(buffer.len() * 2, 0);
                next_out = buffer.as_mut_ptr().offset(offset);
                avail_out = buffer.len().wrapping_add_signed(-offset);
            }
        }
        buffer.truncate(next_out as usize - buffer.as_ptr() as usize);
        self.check_enc_status(status)?;

        unsafe { JxlEncoderReset(self.enc) };
        self.options_ptr = unsafe { JxlEncoderFrameSettingsCreate(self.enc, null()) };

        buffer.shrink_to_fit();
        Ok(buffer)
    }

    // Start encoding
    fn start_encoding<U: PixelType>(&mut self) -> Result<EncoderResult<U>, EncodeError> {
        Ok(EncoderResult {
            data: self._internal()?,
            _pixel_type: PhantomData,
        })
    }
}

// Public interface
impl<'prl, 'mm> JxlEncoder<'prl, 'mm> {
    /// Set a specific encoder frame setting
    ///
    /// # Errors
    /// Return [`EncodeError`] if it fails to set frame option
    pub fn set_frame_option(
        &mut self,
        option: FrameSetting,
        value: i64,
    ) -> Result<(), EncodeError> {
        self.check_enc_status(unsafe {
            JxlEncoderFrameSettingsSetOption(self.options_ptr, option, value)
        })
    }

    /// Return a wrapper type for adding multiple frames to the encoder
    ///
    /// # Errors
    /// Return [`EncodeError`] if it fails to set up the encoder
    pub fn multiple<'enc, U: PixelType>(
        &'enc mut self,
        width: u32,
        height: u32,
    ) -> Result<MultiFrames<'enc, 'prl, 'mm, U>, EncodeError> {
        self.setup_encoder(width, height, U::bits_per_sample(), self.has_alpha)?;
        Ok(MultiFrames::<'enc, 'prl, '_, U>(self, PhantomData))
    }

    /// Encode a JPEG XL image from existing raw JPEG data
    ///
    /// Note: Only support output pixel type of `u8`. Ignore alpha channel settings
    ///
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode_jpeg(&mut self, data: &[u8]) -> Result<EncoderResult<u8>, EncodeError> {
        if let Some(runner) = self.parallel_runner {
            unsafe {
                self.check_enc_status(JxlEncoderSetParallelRunner(
                    self.enc,
                    runner.runner(),
                    runner.as_opaque_ptr(),
                ))?;
            }
        }

        self.set_options()?;

        // If using container format, store JPEG reconstruction metadata
        self.check_enc_status(unsafe { JxlEncoderStoreJPEGMetadata(self.enc, true) })?;

        self.add_jpeg_frame(data)?;
        self.start_encoding()
    }

    /// Encode a JPEG XL image from pixels
    ///
    /// Note: Use RGB(3) channels, native endianness and no alignment. Ignore alpha channel settings
    ///
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode<T: PixelType, U: PixelType>(
        &mut self,
        data: &[T],
        width: u32,
        height: u32,
    ) -> Result<EncoderResult<U>, EncodeError> {
        self.setup_encoder(width, height, U::bits_per_sample(), self.has_alpha)?;
        self.add_frame(&EncoderFrame::new(data))?;
        self.start_encoding::<U>()
    }

    /// Encode a JPEG XL image from pixels with metadata
    ///
    /// Note: Use RGB(3) channels, native endianness and no alignment. Ignore alpha channel settings and remain box uncompressed
    ///
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode_with_metadata<T: PixelType, U: PixelType>(
        &mut self,
        data: &[T],
        width: u32,
        height: u32,
        metadata: Metadata,
    ) -> Result<EncoderResult<U>, EncodeError> {
        self.setup_encoder(width, height, U::bits_per_sample(), self.has_alpha)?;
        self.add_frame(&EncoderFrame::new(data))?;
        self.add_metadata(metadata)?;
        self.start_encoding::<U>()
    }

    /// Encode a JPEG XL image from a frame. See [`EncoderFrame`] for custom options of the original pixels.
    ///
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode_frame<T: PixelType, U: PixelType>(
        &mut self,
        frame: &EncoderFrame<T>,
        width: u32,
        height: u32,
    ) -> Result<EncoderResult<U>, EncodeError> {
        self.setup_encoder(width, height, U::bits_per_sample(), self.has_alpha)?;
        self.add_frame(frame)?;
        self.start_encoding::<U>()
    }
}

impl Drop for JxlEncoder<'_, '_> {
    fn drop(&mut self) {
        unsafe { JxlEncoderDestroy(self.enc) };
    }
}

/// A wrapper type for encoding multiple frames
pub struct MultiFrames<'enc, 'prl, 'mm, U>(&'enc mut JxlEncoder<'prl, 'mm>, PhantomData<U>)
where
    'prl: 'enc,
    'mm: 'enc;

impl<U: PixelType> MultiFrames<'_, '_, '_, U> {
    /// Add a frame to the encoder
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to add a frame
    pub fn add_frame<T: PixelType>(self, frame: &EncoderFrame<T>) -> Result<Self, EncodeError> {
        self.0.add_frame(frame)?;
        Ok(self)
    }

    /// Add a JPEG raw frame to the encoder
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to add a jpeg frame
    pub fn add_jpeg_frame(self, data: &[u8]) -> Result<Self, EncodeError> {
        self.0.add_jpeg_frame(data)?;
        Ok(self)
    }

    /// Encode a JPEG XL image from the frames
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode(self) -> Result<EncoderResult<U>, EncodeError> {
        self.0.start_encoding()
    }
}

/// Return a [`JxlEncoderBuilder`] with default settings
#[must_use]
pub fn encoder_builder<'prl, 'mm>() -> JxlEncoderBuilder<'prl, 'mm> {
    JxlEncoderBuilder::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn test_derive() {
        let e = EncoderSpeed::default().clone();
        println!("{e:?}");

        let e = ColorEncoding::Srgb.clone();
        println!("{e:?}");

        _ = encoder_builder().clone();
    }
}
