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
use std::{marker::PhantomData, ptr::null};

use crate::{
    common::PixelType,
    errors::{check_enc_status, EncodeError},
    memory::JxlMemoryManager,
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

/// JPEG XL Encoder
pub struct JxlEncoder<'a> {
    /// Opaque pointer to the underlying encoder
    enc: *mut jpegxl_sys::JxlEncoder,
    /// Opaque pointer to the encoder options
    options_ptr: *mut JxlEncoderOptions,

    /// Has alpha channel
    has_alpha: bool,

    /// Use container format or not
    use_container: bool,

    /// Color encoding
    color_encoding: Option<JxlColorEncoding>,

    /// Parallel runner
    parallel_runner: Option<&'a dyn JxlParallelRunner>,
}

impl<'a> JxlEncoder<'a> {
    fn new(
        encoder_options: &EncoderOptions,
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
            has_alpha,
            lossless,
            speed,
            quality,
            color_encoding,
            use_container,
        } = *encoder_options;

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

        let mut encoder = Self {
            enc,
            options_ptr,
            has_alpha,
            use_container,
            color_encoding,
            parallel_runner,
        };

        encoder.set_lossless(lossless)?;
        if let Some(s) = speed {
            encoder.set_speed(s)?;
        }
        if let Some(q) = quality {
            encoder.set_quality(q)?;
        }

        encoder.use_container(use_container)?;

        Ok(encoder)
    }

    /// Set lossless mode. Default is lossy
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to set lossless mode
    pub fn set_lossless(&mut self, lossless: bool) -> Result<(), EncodeError> {
        check_enc_status(unsafe { JxlEncoderOptionsSetLossless(self.options_ptr, lossless) })
    }

    /// Set speed
    /// Default: `[EncodeSpeed::Squirrel] (7)`.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to set speed
    pub fn set_speed(&mut self, speed: EncoderSpeed) -> Result<(), EncodeError> {
        check_enc_status(unsafe { JxlEncoderOptionsSetEffort(self.options_ptr, speed as _) })
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
    pub fn set_quality(&mut self, quality: f32) -> Result<(), EncodeError> {
        check_enc_status(unsafe { JxlEncoderOptionsSetDistance(self.options_ptr, quality) })
    }

    /// Configure the encoder to use the JPEG XL container format
    /// Using the JPEG XL container format allows to store metadata such as JPEG reconstruction;
    ///   but it adds a few bytes to the encoded file for container headers even if there is no extra metadata.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to use JPEG XL container format
    pub fn use_container(&mut self, use_container: bool) -> Result<(), EncodeError> {
        check_enc_status(unsafe { JxlEncoderUseContainer(self.enc, use_container) })
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
                check_enc_status(JxlEncoderSetParallelRunner(
                    self.enc,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                ))?
            }
        }

        if let Some(c) = &self.color_encoding {
            unsafe { JxlEncoderSetColorEncoding(self.enc, c) };
        }

        let mut basic_info = unsafe { JxlBasicInfo::new_uninit().assume_init() };
        basic_info.xsize = width;
        basic_info.ysize = height;
        basic_info.uses_original_profile = true as _;

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
        check_enc_status(unsafe { JxlEncoderSetBasicInfo(self.enc, &basic_info) })
    }

    /// Add a frame
    fn add_frame<T: PixelType>(&self, frame: &EncoderFrame<T>) -> Result<(), EncodeError> {
        check_enc_status(unsafe {
            JxlEncoderAddImageFrame(
                self.options_ptr,
                &frame.pixel_format(),
                frame.data.as_ptr().cast(),
                std::mem::size_of_val(frame.data),
            )
        })
    }

    /// Add a frame from JPEG raw data
    fn add_jpeg_frame(&self, data: &[u8]) -> Result<(), EncodeError> {
        check_enc_status(unsafe {
            JxlEncoderAddJPEGFrame(
                self.options_ptr,
                data.as_ptr().cast(),
                std::mem::size_of_val(data),
            )
        })
    }

    /// Start encoding
    #[allow(clippy::cast_sign_loss)]
    fn start_encoding<U: PixelType>(&self) -> Result<Vec<U>, EncodeError> {
        unsafe { JxlEncoderCloseInput(self.enc) };

        let chunk_size = 1024 * 512; // 512 KB is a good initial value
        let mut buffer = vec![U::default(); chunk_size];
        let mut next_out = buffer.as_mut_ptr().cast();
        let mut avail_out = chunk_size;

        let mut status;
        loop {
            status = unsafe { JxlEncoderProcessOutput(self.enc, &mut next_out, &mut avail_out) };

            if status != JxlEncoderStatus::NeedMoreOutput {
                break;
            }

            unsafe {
                let offset = next_out.offset_from(buffer.as_ptr().cast());
                buffer.resize(buffer.len() * 2, U::default());
                next_out = (buffer.as_mut_ptr().cast::<u8>()).offset(offset);
                avail_out = buffer.len() - offset as usize;
            }
        }
        unsafe { buffer.truncate(next_out.offset_from(buffer.as_ptr().cast()) as usize) };
        check_enc_status(status)?;

        unsafe { JxlEncoderReset(self.enc) };

        Ok(buffer)
    }

    /// Encode a JPEG XL image from existing raw JPEG data. Only support output pixel type of `u8`
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode_jpeg(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, EncodeError> {
        // If using container format, store JPEG reconstruction metadata
        check_enc_status(unsafe { JxlEncoderStoreJPEGMetadata(self.enc, self.use_container) })?;

        self.setup_encoder::<u8>(width, height, self.has_alpha)?;
        self.add_jpeg_frame(data)?;
        self.start_encoding()
    }

    /// Encode a JPEG XL image from pixels. Use RGB(3) channels, native endianness and no alignment.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode<T: PixelType, U: PixelType>(
        &self,
        data: &[T],
        width: u32,
        height: u32,
    ) -> Result<Vec<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.has_alpha)?;
        self.add_frame(&EncoderFrame::new(data))?;
        self.start_encoding()
    }

    /// Encode a JPEG XL image from a frame. See [`EncoderFrame`] for custom options of the original pixels.
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode_frame<T: PixelType, U: PixelType>(
        &self,
        frame: &EncoderFrame<T>,
        width: u32,
        height: u32,
    ) -> Result<Vec<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.has_alpha)?;
        self.add_frame(frame)?;
        self.start_encoding()
    }

    /// Return a wrapper type for adding multiple frames to the encoder
    /// # Errors
    /// Return [`EncodeError`] if internal encoder fails to encode
    pub fn multiple<U: PixelType>(
        &self,
        width: u32,
        height: u32,
    ) -> Result<MultiFrames<U>, EncodeError> {
        self.setup_encoder::<U>(width, height, self.has_alpha)?;
        Ok(MultiFrames(self, PhantomData))
    }
}

impl<'a> Drop for JxlEncoder<'a> {
    fn drop(&mut self) {
        unsafe { JxlEncoderDestroy(self.enc) };
    }
}

/// A wrapper type for encoding multiple frames
pub struct MultiFrames<'a, U>(&'a JxlEncoder<'a>, PhantomData<U>);

impl<'a, U: PixelType> MultiFrames<'a, U> {
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
    pub fn encode(&self) -> Result<Vec<U>, EncodeError> {
        self.0.start_encoding()
    }
}

#[derive(Clone)]
struct EncoderOptions {
    has_alpha: bool,
    lossless: bool,
    speed: Option<EncoderSpeed>,
    quality: Option<f32>,
    color_encoding: Option<ColorEncoding>,
    use_container: bool,
}

/// Builder for [`JxlEncoder`]
pub struct JxlEncoderBuilder<'a> {
    options: EncoderOptions,
    memory_manager: Option<&'a dyn JxlMemoryManager>,
    parallel_runner: Option<&'a dyn JxlParallelRunner>,
}

impl<'a> JxlEncoderBuilder<'a> {
    /// Set alpha channel
    pub fn has_alpha(&mut self, f: bool) -> &mut Self {
        self.options.has_alpha = f;
        self
    }

    /// Set lossless
    pub fn lossless(&mut self, f: bool) -> &mut Self {
        self.options.lossless = f;
        self
    }

    /// Set speed
    pub fn speed(&mut self, speed: EncoderSpeed) -> &mut Self {
        self.options.speed = Some(speed);
        self
    }

    /// Set quality
    pub fn quality(&mut self, quality: f32) -> &mut Self {
        self.options.quality = Some(quality);
        self
    }

    /// Set color encoding
    pub fn color_encoding(&mut self, encoding: ColorEncoding) -> &mut Self {
        self.options.color_encoding = Some(encoding);
        self
    }

    /// Use container or not
    pub fn use_container(&mut self, use_container: bool) -> &mut Self {
        self.options.use_container = use_container;
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

    /// Consume the builder and get the encoder
    /// # Errors
    /// Return [`EncodeError::CannotCreateEncoder`] if it fails to create the encoder
    pub fn build(&mut self) -> Result<JxlEncoder<'a>, EncodeError> {
        JxlEncoder::new(
            &self.options,
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
            speed: None,
            quality: None,
            color_encoding: None,
            use_container: false,
        },
        memory_manager: None,
        parallel_runner: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{decoder_builder, JxlDecoderResult, ThreadsRunner};

    use image::io::Reader as ImageReader;
    #[test]
    fn test_encode() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let parallel_runner = ThreadsRunner::default();
        let encoder = encoder_builder()
            .parallel_runner(&parallel_runner)
            .speed(EncoderSpeed::Falcon)
            .build()?;

        let result: Vec<f32> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        let mut decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build()?;
        let _res: JxlDecoderResult<f32> = decoder.decode(unsafe {
            std::slice::from_raw_parts(
                result.as_ptr().cast(),
                result.len() * std::mem::size_of::<f32>(),
            )
        })?;

        Ok(())
    }

    #[test]
    fn test_encode_jpeg() -> Result<(), image::ImageError> {
        let sample = std::fs::read("test/sample.jpg")?;
        let parallel_runner = ThreadsRunner::default();
        let encoder = encoder_builder()
            .speed(EncoderSpeed::Falcon)
            .use_container(true)
            .parallel_runner(&parallel_runner)
            .build()?;

        encoder.encode_jpeg(&sample, 100, 100)?;

        Ok(())
    }

    #[test]
    fn test_encoder_builder() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba8();
        let parallel_runner = ThreadsRunner::default();
        let encoder = encoder_builder()
            .has_alpha(true)
            .lossless(false)
            .speed(EncoderSpeed::Falcon)
            .quality(3.0)
            .color_encoding(ColorEncoding::LinearSRgb)
            .parallel_runner(&parallel_runner)
            .build()?;

        let _res: Vec<u8> = encoder.encode_frame(
            &EncoderFrame::new(sample.as_raw()).num_channels(4),
            sample.width(),
            sample.height(),
        )?;

        Ok(())
    }

    #[test]
    fn test_encode_multi_frames() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let sample_jpeg = std::fs::read("test/sample.jpg")?;
        let parallel_runner = ThreadsRunner::default();
        let encoder = encoder_builder()
            .parallel_runner(&parallel_runner)
            .build()?;

        let _res: Vec<u8> = encoder
            .multiple(sample.width(), sample.height())?
            .add_frame(&EncoderFrame::new(sample.as_raw()))?
            .add_jpeg_frame(&sample_jpeg)?
            .encode()?;

        Ok(())
    }

    #[test]
    fn test_memory_manager() -> Result<(), Box<dyn std::error::Error>> {
        use crate::memory::MallocManager;

        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
        let memory_manager = MallocManager::default();
        let parallel_runner = ThreadsRunner::default();

        let encoder = encoder_builder()
            .speed(EncoderSpeed::Falcon)
            .parallel_runner(&parallel_runner)
            .memory_manager(&memory_manager)
            .build()?;
        let custom_buffer: Vec<u8> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        let encoder = encoder_builder()
            .parallel_runner(&parallel_runner)
            .speed(EncoderSpeed::Falcon)
            .build()?;
        let default_buffer: Vec<u8> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        assert!(
            custom_buffer == default_buffer,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
