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
use std::{cell::Cell, marker::PhantomData, ops::Deref, ptr::null};

use crate::{
    common::PixelType, errors::check_enc_status, errors::EncodeError, memory::JxlMemoryManager,
    parallel::JxlParallelRunner,
};

/// Encoding speed, default at Squirrel(7)
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

/// Encoding color profile
#[derive(Debug, Clone, Copy)]
pub enum ColorEncoding {
    /// SRGB, default for uint pixel types
    SRgb,
    /// Linear SRGB, default for float pixel types
    LinearSRgb,
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
pub struct JxlEncoder<'a> {
    /// Opaque pointer to the underlying encoder
    enc: *mut jpegxl_sys::JxlEncoder,
    /// Opaque pointer to the encoder options
    options_ptr: Cell<*mut JxlEncoderOptions>,

    /// Color encoding
    color_encoding: Option<JxlColorEncoding>,

    /// Various options
    options: EncoderOptions,

    /// Parallel runner
    parallel_runner: Option<&'a dyn JxlParallelRunner>,
}

impl<'a> JxlEncoder<'a> {
    fn new(
        encoder_options: EncoderOptions,
        memory_manager: Option<jpegxl_sys::JxlMemoryManager>,
        parallel_runner: Option<&'a dyn JxlParallelRunner>,
    ) -> Result<Self, EncodeError> {
        let enc = unsafe {
            memory_manager.map_or_else(
                || JxlEncoderCreate(null()),
                |memory_manager| JxlEncoderCreate(&memory_manager),
            )
        };

        if enc.is_null() {
            return Err(EncodeError::CannotCreateEncoder);
        }

        let options_ptr = unsafe { JxlEncoderOptionsCreate(enc, null()) };

        let EncoderOptions {
            lossless,
            speed,
            quality,
            decoding_speed,
            color_encoding,
            ..
        } = encoder_options;

        let color_encoding = color_encoding.map(|c| unsafe {
            let mut color_encoding = JxlColorEncoding::new_uninit();
            match c {
                ColorEncoding::SRgb => {
                    JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), false)
                }
                ColorEncoding::LinearSRgb => {
                    JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), false)
                }
            }
            color_encoding.assume_init()
        });

        let encoder = Self {
            enc,
            options_ptr: Cell::new(options_ptr),
            color_encoding,
            options: encoder_options,
            parallel_runner,
        };

        encoder.set_lossless(lossless)?;
        encoder.set_speed(speed)?;
        encoder.set_quality(quality)?;
        encoder.decoding_speed(decoding_speed)?;

        Ok(encoder)
    }

    /// Set if have alpha channel
    pub fn has_alpha(&mut self, value: bool) {
        self.options.has_alpha = value;
    }

    /// Configure the encoder to use the JPEG XL container format
    /// Using the JPEG XL container format allows to store metadata such as JPEG reconstruction;
    ///   but it adds a few bytes to the encoded file for container headers even if there is no extra metadata.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to use JPEG XL container format
    pub fn use_container(&self, use_container: bool) -> Result<(), EncodeError> {
        check_enc_status(
            unsafe { JxlEncoderUseContainer(self.enc, use_container) },
            "If use container",
        )
    }

    /// Set lossless mode. Default is lossy
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to set lossless mode
    pub fn set_lossless(&self, lossless: bool) -> Result<(), EncodeError> {
        check_enc_status(
            unsafe { JxlEncoderOptionsSetLossless(self.options_ptr.get(), lossless) },
            "Set lossless",
        )
    }

    /// Set speed
    /// Default: `[EncodeSpeed::Squirrel] (7)`.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to set speed
    pub fn set_speed(&self, speed: EncoderSpeed) -> Result<(), EncodeError> {
        check_enc_status(
            unsafe { JxlEncoderOptionsSetEffort(self.options_ptr.get(), speed as _) },
            "Set speed",
        )
    }

    /// Set quality for lossy compression: target max butteraugli distance, lower = higher quality
    ///  Range: 0 .. 15.<br />
    ///    0.0 = mathematically lossless (however, use `set_lossless` to use true lossless). <br />
    ///    1.0 = visually lossless. <br />
    ///    Recommended range: 0.5 .. 3.0. <br />
    ///    Default value: 1.0. <br />
    ///    If `set_lossless` is used, this value is unused and implied to be 0.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to set quality
    pub fn set_quality(&self, quality: f32) -> Result<(), EncodeError> {
        check_enc_status(
            unsafe { JxlEncoderOptionsSetDistance(self.options_ptr.get(), quality) },
            "Set quality",
        )
    }

    /// Set the decoding speed tier for the provided options.
    /// Minimum is 0 (highest quality), and maximum is 4 (lowest quality). Default is 0.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to set decoding speed
    pub fn decoding_speed(&self, value: i32) -> Result<(), EncodeError> {
        check_enc_status(
            unsafe { JxlEncoderOptionsSetDecodingSpeed(self.options_ptr.get(), value) },
            "Set decoding speed",
        )
    }

    /// Encode a JPEG XL image from existing raw JPEG data
    ///
    /// Note: Only support output pixel type of `u8`. Ignore alpha channel settings
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
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode<T: PixelType, U: PixelType>(
        &self,
        data: &[T],
        width: u32,
        height: u32,
    ) -> Result<EncoderResult<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.options.has_alpha)?;
        self.add_frame(&EncoderFrame::new(data))?;
        self.start_encoding::<U>()
    }

    /// Encode a JPEG XL image from a frame. See [`EncoderFrame`] for custom options of the original pixels.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode_frame<T: PixelType, U: PixelType>(
        &self,
        frame: &EncoderFrame<T>,
        width: u32,
        height: u32,
    ) -> Result<EncoderResult<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.options.has_alpha)?;
        self.add_frame(frame)?;
        self.start_encoding::<U>()
    }

    /// Return a wrapper type for adding multiple frames to the encoder
    /// # Errors
    /// Return [`EncodeError`] if it fails to set up the encoder
    pub fn multiple<U: PixelType>(
        &self,
        width: u32,
        height: u32,
    ) -> Result<MultiFrames<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.options.has_alpha)?;
        Ok(MultiFrames(self, PhantomData))
    }

    /// Setup the encoder
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

        self.use_container(self.options.use_container)?;

        if let Some(c) = &self.color_encoding {
            unsafe { JxlEncoderSetColorEncoding(self.enc, c) };
        }

        let mut basic_info = unsafe { JxlBasicInfo::new_uninit().assume_init() };
        basic_info.xsize = width;
        basic_info.ysize = height;
        basic_info.uses_original_profile = true as _;
        basic_info.have_container = self.options.use_container as _;

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

    /// Add a frame
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

    /// Add a frame from JPEG raw data
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

    /// Start encoding
    #[allow(clippy::cast_sign_loss)]
    fn start_encoding<U: PixelType>(&self) -> Result<EncoderResult<U>, EncodeError> {
        unsafe { JxlEncoderCloseInput(self.enc) };

        let mut buffer = vec![0; self.options.init_buffer_size];
        let mut next_out = buffer.as_mut_ptr().cast();
        let mut avail_out = self.options.init_buffer_size;

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
            .set(unsafe { JxlEncoderOptionsCreate(self.enc, self.options_ptr.get()) });

        buffer.shrink_to_fit();
        Ok(EncoderResult {
            data: buffer,
            _output_pixel_type: PhantomData,
        })
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
    pub fn add_frame<T: PixelType>(&self, frame: &EncoderFrame<T>) -> Result<&Self, EncodeError> {
        self.0.add_frame(frame)?;
        Ok(self)
    }

    /// Add a JPEG raw frame to the encoder
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to add a jpeg frame
    pub fn add_jpeg_frame(&self, data: &[u8]) -> Result<&Self, EncodeError> {
        self.0.add_jpeg_frame(data)?;
        Ok(self)
    }

    /// Encode a JPEG XL image from the frames
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode(&self) -> Result<EncoderResult<U>, EncodeError> {
        self.0.start_encoding()
    }
}

#[derive(Clone)]
struct EncoderOptions {
    has_alpha: bool,
    lossless: bool,
    speed: EncoderSpeed,
    quality: f32,
    color_encoding: Option<ColorEncoding>,
    use_container: bool,
    decoding_speed: i32,
    init_buffer_size: usize,
}

/// Builder for [`JxlEncoder`]
pub struct JxlEncoderBuilder<'a> {
    options: EncoderOptions,
    memory_manager: Option<&'a dyn JxlMemoryManager>,
    parallel_runner: Option<&'a dyn JxlParallelRunner>,
}

impl<'a> JxlEncoderBuilder<'a> {
    /// Set alpha channel
    pub fn has_alpha(&mut self, value: bool) -> &mut Self {
        self.options.has_alpha = value;
        self
    }

    /// Set lossless
    pub fn lossless(&mut self, value: bool) -> &mut Self {
        self.options.lossless = value;
        self
    }

    /// Set speed
    pub fn speed(&mut self, value: EncoderSpeed) -> &mut Self {
        self.options.speed = value;
        self
    }

    /// Set quality
    pub fn quality(&mut self, value: f32) -> &mut Self {
        self.options.quality = value;
        self
    }

    /// Set color encoding
    pub fn color_encoding(&mut self, value: ColorEncoding) -> &mut Self {
        self.options.color_encoding = Some(value);
        self
    }

    /// Set decoding speed
    ///
    /// Range: 0 ..= 4
    pub fn decoding_speed(&mut self, value: i32) -> &mut Self {
        self.options.decoding_speed = value;
        self
    }

    /// Use container or not
    pub fn use_container(&mut self, value: bool) -> &mut Self {
        self.options.use_container = value;
        self
    }

    /// Set initial buffer size in bytes.
    ///
    /// Default: 1MiB
    pub fn init_buffer_size(&mut self, value: usize) -> &mut Self {
        self.options.init_buffer_size = value;
        self
    }

    /// Set memory manager
    pub fn memory_manager(&mut self, value: &'a dyn JxlMemoryManager) -> &mut Self {
        self.memory_manager = Some(value);
        self
    }

    /// Set parallel runner
    pub fn parallel_runner(&mut self, value: &'a dyn JxlParallelRunner) -> &mut Self {
        self.parallel_runner = Some(value);
        self
    }

    /// Consume the builder and get the encoder
    /// # Errors
    /// Return [`EncodeError::CannotCreateEncoder`] if it fails to create the encoder
    pub fn build(&mut self) -> Result<JxlEncoder<'a>, EncodeError> {
        JxlEncoder::new(
            self.options.clone(),
            self.memory_manager.map(|m| m.to_manager()),
            self.parallel_runner,
        )
    }
}

/// Return a [`JxlEncoderBuilder`] with default settings
#[must_use]
pub fn encoder_builder<'a>() -> JxlEncoderBuilder<'a> {
    JxlEncoderBuilder {
        options: EncoderOptions {
            has_alpha: false,
            lossless: false,
            speed: EncoderSpeed::Squirrel,
            quality: 1.0,
            color_encoding: None,
            use_container: false,
            decoding_speed: 0,
            init_buffer_size: 1024 * 1024,
        },
        memory_manager: None,
        parallel_runner: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{decode::DecoderResult, decoder_builder, Endianness, ThreadsRunner};

    use image::{io::Reader as ImageReader, GenericImageView, ImageFormat};

    type Res = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_encode_single() -> Res {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let parallel_runner = ThreadsRunner::default();
        let encoder = encoder_builder()
            .parallel_runner(&parallel_runner)
            .build()?;

        let result: EncoderResult<u16> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build()?;
        let _res: DecoderResult<f32> = decoder.decode(&result)?;

        Ok(())
    }

    #[test]
    fn test_encode_jpeg() -> Res {
        let sample = std::fs::read("test/sample.jpg")?;
        let jpeg =
            ImageReader::with_format(std::io::Cursor::new(&sample), ImageFormat::Jpeg).decode()?;

        let parallel_runner = ThreadsRunner::default();
        let encoder = encoder_builder()
            .use_container(true)
            .parallel_runner(&parallel_runner)
            .build()?;

        let _res = encoder.encode_jpeg(&sample, jpeg.width(), jpeg.height())?;

        Ok(())
    }

    #[test]
    fn test_encoder_builder() -> Res {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba8();
        let parallel_runner = ThreadsRunner::default();
        let mut encoder = encoder_builder()
            .has_alpha(true)
            .lossless(false)
            .speed(EncoderSpeed::Tortoise)
            .quality(3.0)
            .color_encoding(ColorEncoding::LinearSRgb)
            .decoding_speed(4)
            .init_buffer_size(64)
            .parallel_runner(&parallel_runner)
            .build()?;

        let _res: EncoderResult<u8> = encoder.encode_frame(
            &EncoderFrame::new(sample.as_raw()).num_channels(4),
            sample.width(),
            sample.height(),
        )?;

        // Check encoder reset
        encoder.has_alpha(false);
        let _res: EncoderResult<u8> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        Ok(())
    }

    #[test]
    fn test_encode_multi_frames() -> Res {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let sample_jpeg = std::fs::read("test/sample.jpg")?;
        let parallel_runner = ThreadsRunner::default();
        let encoder = encoder_builder()
            .color_encoding(ColorEncoding::SRgb)
            .parallel_runner(&parallel_runner)
            .build()?;

        let frame = EncoderFrame::new(sample.as_raw())
            .endianness(Endianness::Native)
            .align(0);

        let _res: EncoderResult<u8> = encoder
            .multiple(sample.width(), sample.height())?
            .add_frame(&frame)?
            .add_jpeg_frame(&sample_jpeg)?
            .encode()?;

        Ok(())
    }

    #[test]
    fn test_invalid_data() -> Res {
        let encoder = encoder_builder().has_alpha(true).build()?;
        assert!(matches!(
            encoder.encode::<u8, u8>(&[], 0, 0),
            Err(EncodeError::GenericError("Set basic info"))
        ));

        assert!(matches!(
            encoder.encode::<f32, f32>(&[1.0, 1.0, 1.0, 0.5], 1, 1),
            Err(EncodeError::NotSupported)
        ));

        Ok(())
    }
}
