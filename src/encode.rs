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

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::*;
use std::{cell::Cell, marker::PhantomData, ops::Deref, pin::Pin, ptr::null};

use crate::{
    common::PixelType, errors::check_enc_status, errors::EncodeError, memory::JxlMemoryManager,
    parallel::JxlParallelRunner,
};

/// Encoding speed
#[derive(Debug, Clone, Copy)]
pub enum EncoderSpeed {
    /// Fastest, 3
    Falcon = 3,
    /// 4
    Cheetah,
    /// 5
    Hare,
    /// 6
    Wombat,
    /// 7
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
#[derive(Clone)]
pub enum ColorEncoding {
    /// SRGB, default for uint pixel types
    SRgb,
    /// Linear SRGB, default for float pixel types
    LinearSRgb,
}

impl From<ColorEncoding> for JxlColorEncoding {
    fn from(val: ColorEncoding) -> Self {
        use ColorEncoding::*;

        let mut color_encoding = JxlColorEncoding::new_uninit();

        unsafe {
            match val {
                SRgb => JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), false),
                LinearSRgb => JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), false),
            }
            color_encoding.assume_init()
        }
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
    /// Create a default frame from the data. Use RGB(3) channels, native endianness and no alignment.
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
    _output_pixel_type: PhantomData<U>,
}

impl<U: PixelType> Deref for EncoderResult<U> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data.as_ref()
    }
}

/// JPEG XL Encoder
#[derive(Builder)]
#[builder(build_fn(skip))]
#[builder(setter(strip_option))]
pub struct JxlEncoder<'a> {
    /// Opaque pointer to the underlying encoder
    #[builder(setter(skip))]
    enc: *mut jpegxl_sys::JxlEncoder,
    /// Opaque pointer to the encoder options
    #[builder(setter(skip))]
    options_ptr: Cell<*mut JxlEncoderOptions>,

    /// Set alpha channel
    ///
    /// Default: false
    has_alpha: bool,
    /// Set lossless
    ///
    /// Default: false
    lossless: bool,
    /// Set speed
    ///
    /// Default: `[EncodeSpeed::Squirrel] (7)`.
    speed: EncoderSpeed,
    /// Set quality for lossy compression: target max butteraugli distance, lower = higher quality
    ///
    ///  Range: 0 .. 15.<br />
    ///    0.0 = mathematically lossless (however, use `lossless` to use true lossless). <br />
    ///    1.0 = visually lossless. <br />
    ///    Recommended range: 0.5 .. 3.0. <br />
    ///    Default value: 1.0. <br />
    ///    If `lossless` is set to `true`, this value is unused and implied to be 0.
    quality: f32,
    /// Configure the encoder to use the JPEG XL container format
    ///
    /// Using the JPEG XL container format allows to store metadata such as JPEG reconstruction;
    /// but it adds a few bytes to the encoded file for container headers even if there is no extra metadata.
    use_container: bool,
    /// Set the decoding speed tier
    ///
    /// Minimum is 0 (highest quality), and maximum is 4 (lowest quality). Default is 0.
    decoding_speed: i32,
    /// Set initial output buffer size in bytes
    ///
    /// Default: 1 MiB
    init_buffer_size: usize,

    /// Set color encoding
    ///
    /// Default: determined by input data type
    #[builder(setter(into))]
    color_encoding: Option<JxlColorEncoding>,

    /// Set parallel runner
    ///
    /// Default: `None`, indicating single thread execution
    parallel_runner: Option<&'a dyn JxlParallelRunner>,
}

impl<'a> JxlEncoderBuilder<'a> {
    fn _build<'b>(
        &self,
        memory_manager: Option<Pin<&'b dyn JxlMemoryManager>>,
    ) -> Result<JxlEncoder<'a>, EncodeError> {
        let enc = unsafe {
            memory_manager.map_or_else(
                || JxlEncoderCreate(null()),
                |memory_manager| JxlEncoderCreate(&memory_manager.manager()),
            )
        };

        if enc.is_null() {
            return Err(EncodeError::CannotCreateEncoder);
        }

        let options_ptr = unsafe { JxlEncoderOptionsCreate(enc, null()) };

        let encoder = JxlEncoder {
            enc,
            options_ptr: Cell::new(options_ptr),
            has_alpha: self.has_alpha.unwrap_or_default(),
            lossless: self.lossless.unwrap_or_default(),
            speed: self.speed.unwrap_or_default(),
            quality: self.quality.unwrap_or(1.0),
            use_container: self.use_container.unwrap_or_default(),
            decoding_speed: self.decoding_speed.unwrap_or_default(),
            init_buffer_size: self.init_buffer_size.unwrap_or(1024 * 1024),
            color_encoding: self.color_encoding.clone().flatten(),
            parallel_runner: self.parallel_runner.flatten(),
        };

        Ok(encoder)
    }

    /// Build a [`JxlEncoder`]
    ///
    /// # Errors
    /// Return [`EncodeError::CannotCreateEncoder`] if it fails to create the encoder
    pub fn build(&self) -> Result<JxlEncoder<'a>, EncodeError> {
        Self::_build(self, None)
    }

    /// Build a [`JxlEncoder`] with custom memory manager
    ///
    /// # Errors
    /// Return [`EncodeError::CannotCreateEncoder`] if it fails to create the encoder
    pub fn build_with<'b>(
        &self,
        memory_manager: Pin<&'b dyn JxlMemoryManager>,
    ) -> Result<JxlEncoder<'a>, EncodeError> {
        Self::_build(self, Some(memory_manager))
    }
}

// Private helper functions
impl<'a> JxlEncoder<'a> {
    // Set options
    fn set_options(&self) -> Result<(), EncodeError> {
        check_enc_status(
            unsafe { JxlEncoderUseContainer(self.enc, self.use_container) },
            "If use container",
        )?;
        check_enc_status(
            unsafe { JxlEncoderOptionsSetLossless(self.options_ptr.get(), self.lossless) },
            "Set lossless",
        )?;
        check_enc_status(
            unsafe { JxlEncoderOptionsSetEffort(self.options_ptr.get(), self.speed as _) },
            "Set speed",
        )?;
        check_enc_status(
            unsafe { JxlEncoderOptionsSetDistance(self.options_ptr.get(), self.quality) },
            "Set quality",
        )?;
        check_enc_status(
            unsafe {
                JxlEncoderOptionsSetDecodingSpeed(self.options_ptr.get(), self.decoding_speed)
            },
            "Set decoding speed",
        )?;

        Ok(())
    }

    // Setup the encoder
    fn setup_encoder<U: PixelType>(
        &self,
        width: u32,
        height: u32,
        has_alpha: bool,
    ) -> Result<(), EncodeError> {
        if let Some(runner) = self.parallel_runner {
            unsafe {
                check_enc_status(
                    JxlEncoderSetParallelRunner(self.enc, runner.runner(), runner.as_opaque_ptr()),
                    "Set parallel runner",
                )?
            }
        }

        self.set_options()?;

        if let Some(c) = &self.color_encoding {
            unsafe { JxlEncoderSetColorEncoding(self.enc, c) };
        }

        let mut basic_info = unsafe { JxlBasicInfo::new_uninit().assume_init() };
        basic_info.xsize = width;
        basic_info.ysize = height;
        basic_info.uses_original_profile = true as _;
        basic_info.have_container = self.use_container as _;

        let (bits, exp) = U::bits_per_sample();
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
        check_enc_status(
            unsafe { JxlEncoderSetBasicInfo(self.enc, &basic_info) },
            "Set basic info",
        )
    }

    // Add a frame
    fn add_frame<T: PixelType>(&self, frame: &EncoderFrame<T>) -> Result<(), EncodeError> {
        check_enc_status(
            unsafe {
                JxlEncoderAddImageFrame(
                    self.options_ptr.get(),
                    &frame.pixel_format(),
                    frame.data.as_ptr().cast(),
                    std::mem::size_of_val(frame.data),
                )
            },
            "Add frame",
        )
    }

    // Add a frame from JPEG raw data
    fn add_jpeg_frame(&self, data: &[u8]) -> Result<(), EncodeError> {
        check_enc_status(
            unsafe {
                JxlEncoderAddJPEGFrame(
                    self.options_ptr.get(),
                    data.as_ptr().cast(),
                    std::mem::size_of_val(data),
                )
            },
            "Add JPEG frame",
        )
    }

    // Start encoding
    #[allow(clippy::cast_sign_loss)]
    fn start_encoding<U: PixelType>(&self) -> Result<EncoderResult<U>, EncodeError> {
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
                buffer.resize(buffer.len() * 2, 0);
                next_out = (buffer.as_mut_ptr()).offset(offset);
                avail_out = buffer.len() - offset as usize;
            }
        }
        unsafe { buffer.truncate(next_out.offset_from(buffer.as_ptr()) as usize) };
        check_enc_status(status, "Process output")?;

        unsafe { JxlEncoderReset(self.enc) };
        self.options_ptr
            .set(unsafe { JxlEncoderOptionsCreate(self.enc, null()) });

        buffer.shrink_to_fit();
        Ok(EncoderResult {
            data: buffer,
            _output_pixel_type: PhantomData,
        })
    }
}

// Public interface
impl<'a> JxlEncoder<'a> {
    /// Set if result has alpha channel
    pub fn has_alpha(&mut self, value: bool) {
        self.has_alpha = value;
    }

    /// Configure the encoder to use the JPEG XL container format
    ///
    /// Using the JPEG XL container format allows to store metadata such as JPEG reconstruction;
    /// but it adds a few bytes to the encoded file for container headers even if there is no extra metadata
    pub fn use_container(&mut self, use_container: bool) {
        self.use_container = use_container;
    }

    /// Set lossless mode. Default is lossy
    pub fn lossless(&mut self, lossless: bool) {
        self.lossless = lossless;
    }

    /// Set speed
    ///
    /// Default: `[EncodeSpeed::Squirrel] (7)`
    pub fn speed(&mut self, speed: EncoderSpeed) {
        self.speed = speed;
    }

    /// Set quality for lossy compression: target max butteraugli distance, lower = higher quality
    ///
    ///  Range: 0 .. 15.<br />
    ///    0.0 = mathematically lossless (however, use `set_lossless` to use true lossless). <br />
    ///    1.0 = visually lossless. <br />
    ///    Recommended range: 0.5 .. 3.0. <br />
    ///    Default value: 1.0. <br />
    ///    If `lossless` is set to `true`, this value is unused and implied to be 0
    pub fn quality(&mut self, quality: f32) {
        self.quality = quality;
    }

    /// Set the decoding speed tier for the provided options.
    ///
    /// Minimum is 0 (highest quality), and maximum is 4 (lowest quality). Default is 0
    pub fn decoding_speed(&mut self, value: i32) {
        self.decoding_speed = value;
    }

    /// Return a wrapper type for adding multiple frames to the encoder
    ///
    /// # Errors
    /// Return [`EncodeError`] if it fails to set up the encoder
    pub fn multiple<U: PixelType>(
        &self,
        width: u32,
        height: u32,
    ) -> Result<MultiFrames<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.has_alpha)?;
        Ok(MultiFrames(self, PhantomData))
    }

    /// Encode a JPEG XL image from existing raw JPEG data
    ///
    /// Note: Only support output pixel type of `u8`. Ignore alpha channel settings
    ///
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode_jpeg(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<EncoderResult<u8>, EncodeError> {
        // If using container format, store JPEG reconstruction metadata
        check_enc_status(
            unsafe { JxlEncoderStoreJPEGMetadata(self.enc, true) },
            "Set store jpeg metadata",
        )?;

        self.setup_encoder::<u8>(width, height, false)?;
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
        &self,
        data: &[T],
        width: u32,
        height: u32,
    ) -> Result<EncoderResult<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.has_alpha)?;
        self.add_frame(&EncoderFrame::new(data))?;
        self.start_encoding::<U>()
    }

    /// Encode a JPEG XL image from a frame. See [`EncoderFrame`] for custom options of the original pixels.
    ///
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode_frame<T: PixelType, U: PixelType>(
        &self,
        frame: &EncoderFrame<T>,
        width: u32,
        height: u32,
    ) -> Result<EncoderResult<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.has_alpha)?;
        self.add_frame(frame)?;
        self.start_encoding::<U>()
    }
}

impl<'a> Drop for JxlEncoder<'a> {
    fn drop(&mut self) {
        unsafe { JxlEncoderDestroy(self.enc) };
    }
}

/// A wrapper type for encoding multiple frames
pub struct MultiFrames<'a, 'b, U>(&'a JxlEncoder<'b>, PhantomData<U>);

impl<'a, 'b, U: PixelType> MultiFrames<'a, 'b, U> {
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
pub fn encoder_builder<'a>() -> JxlEncoderBuilder<'a> {
    JxlEncoderBuilder::default()
}
