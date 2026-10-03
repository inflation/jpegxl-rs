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

use std::{mem::MaybeUninit, ptr::null};

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::encoder::encode::*;

use super::{ColorEncoding, EncoderFrame, FrameSettings, ImageInfo, JxlEncoder, Metadata};
use crate::{common::PixelType, errors::EncodeError};

/// An encoding session that is driven by the caller.
///
/// Add frames and boxes, then [`finish`](Self::finish) to get the encoded image.
///
/// ```
/// # use jpegxl_rs::{encoder_builder, encode::{EncoderFrame, ImageInfo}};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pixels = vec![0u8; 8 * 8 * 3];
/// let mut encoder = encoder_builder().build()?;
/// let mut session = encoder.session(&ImageInfo::builder().width(8).height(8).build())?;
/// session.add_frame(&EncoderFrame::new(&pixels))?;
/// let data = session.finish()?;
/// # Ok(()) }
/// ```
///
/// Dropping the session resets the encoder, so it can be reused.
pub struct Session<'enc, 'prl, 'mm> {
    enc: &'enc mut JxlEncoder<'prl, 'mm>,
    output: Vec<u8>,
    last_frame: LastFrame,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LastFrame {
    None,
    Queued,
    /// Written by [`Session::take_output`] as a non-final frame, so the image cannot be finished
    Taken,
}

impl<'enc, 'prl, 'mm> Session<'enc, 'prl, 'mm> {
    pub(crate) fn new(
        enc: &'enc mut JxlEncoder<'prl, 'mm>,
        info: Option<&ImageInfo>,
    ) -> Result<Self, EncodeError> {
        let session = Self {
            enc,
            output: Vec::new(),
            last_frame: LastFrame::None,
        };
        session.setup(info)?;
        for (t, data, compress) in &session.enc.boxes {
            session.add_box(*t, data, *compress)?;
        }
        Ok(session)
    }

    fn setup(&self, info: Option<&ImageInfo>) -> Result<(), EncodeError> {
        let enc = &*self.enc;
        if let Some(runner) = enc.parallel_runner {
            // SAFETY: `enc.enc` is valid until drop and the runner outlives it
            enc.check_enc_status(unsafe {
                JxlEncoderSetParallelRunner(enc.enc, runner.runner(), runner.as_opaque_ptr())
            })?;
        }
        // SAFETY: `enc.enc` is valid until drop
        enc.check_enc_status(unsafe { JxlEncoderUseContainer(enc.enc, enc.use_container.into()) })?;
        if enc.use_box {
            // SAFETY: `enc.enc` is valid until drop
            enc.check_enc_status(unsafe { JxlEncoderUseBoxes(enc.enc) })?;
        }

        let Some(info) = info else {
            return Ok(());
        };

        // SAFETY: `JxlEncoderInitBasicInfo` initializes `info`
        let mut basic_info = unsafe {
            let mut info = MaybeUninit::uninit();
            JxlEncoderInitBasicInfo(info.as_mut_ptr());
            info.assume_init()
        };
        basic_info.xsize = info.width;
        basic_info.ysize = info.height;
        basic_info.have_container = enc.use_container.into();
        basic_info.uses_original_profile = enc.uses_original_profile.into();
        basic_info.bits_per_sample = info.bits_per_sample;
        basic_info.exponent_bits_per_sample = info.exponent_bits_per_sample;
        if info.has_alpha {
            basic_info.num_extra_channels = 1;
            basic_info.alpha_bits = info.bits_per_sample;
            basic_info.alpha_exponent_bits = info.exponent_bits_per_sample;
        }
        if let Some(ColorEncoding::SrgbLuma | ColorEncoding::LinearSrgbLuma) = enc.color_encoding {
            basic_info.num_color_channels = 1;
        }
        if let Some(target_intensity) = enc.target_intensity {
            basic_info.intensity_target = target_intensity;
        }
        if let Some(runner) = enc.parallel_runner {
            runner.callback_basic_info(&basic_info);
        }

        // SAFETY: `enc.enc` is valid until drop
        enc.check_enc_status(unsafe { JxlEncoderSetBasicInfo(enc.enc, &raw const basic_info) })?;
        if let Some(color_encoding) = &enc.color_encoding {
            // SAFETY: `enc.enc` is valid until drop
            enc.check_enc_status(unsafe {
                JxlEncoderSetColorEncoding(enc.enc, &color_encoding.into())
            })?;
        }
        Ok(())
    }

    /// Add a metadata box
    ///
    /// # Errors
    /// Return [`EncodeError`] if it fails to add the box
    pub fn add_metadata(&mut self, metadata: &Metadata, compress: bool) -> Result<(), EncodeError> {
        let (t, data) = metadata.parts();
        self.add_box(t, data, compress)
    }

    fn add_box(&self, t: [u8; 4], data: &[u8], compress: bool) -> Result<(), EncodeError> {
        let enc = &*self.enc;
        // SAFETY: `enc.enc` is valid until drop
        enc.check_enc_status(unsafe { JxlEncoderUseBoxes(enc.enc) })?;
        // SAFETY: `enc.enc` is valid until drop
        enc.check_enc_status(unsafe {
            JxlEncoderAddBox(
                enc.enc,
                &Metadata::box_type(t),
                data.as_ptr().cast(),
                data.len(),
                compress.into(),
            )
        })
    }

    /// The settings a frame uses unless it has its own,
    /// see [`JxlEncoder::frame_settings`]
    #[must_use]
    pub fn frame_settings(&self) -> FrameSettings {
        self.enc.frame_settings()
    }

    /// Create frame settings, owned by the encoder until it is reset
    fn create_frame_settings(
        &self,
        settings: &FrameSettings,
    ) -> Result<*mut JxlEncoderFrameSettings, EncodeError> {
        let enc = &*self.enc;
        // SAFETY: `enc.enc` is valid until drop
        let ptr = unsafe { JxlEncoderFrameSettingsCreate(enc.enc, null()) };
        if ptr.is_null() {
            return Err(EncodeError::OutOfMemory);
        }

        let set = |id, value| {
            // SAFETY: `ptr` is valid until the encoder is reset
            enc.check_enc_status(unsafe { JxlEncoderFrameSettingsSetOption(ptr, id, value) })
        };
        if let Some(lossless) = settings.lossless {
            // SAFETY: `ptr` is valid until the encoder is reset
            enc.check_enc_status(unsafe { JxlEncoderSetFrameLossless(ptr, lossless.into()) })?;
        }
        set(JxlEncoderFrameSettingId::Effort, settings.speed as _)?;
        // SAFETY: `ptr` is valid until the encoder is reset
        enc.check_enc_status(unsafe { JxlEncoderSetFrameDistance(ptr, settings.quality) })?;
        set(
            JxlEncoderFrameSettingId::DecodingSpeed,
            settings.decoding_speed,
        )?;
        for &(id, value) in &settings.options {
            set(id, value)?;
        }
        Ok(ptr)
    }

    /// Add a frame of pixels
    ///
    /// # Errors
    /// Return [`EncodeError`] if the frame is invalid
    pub fn add_frame<T: PixelType>(&mut self, frame: &EncoderFrame<T>) -> Result<(), EncodeError> {
        let settings = match frame.settings {
            Some(settings) => self.create_frame_settings(settings)?,
            None => self.create_frame_settings(&self.frame_settings())?,
        };
        // SAFETY: `settings` is valid and the size matches `frame.data`
        self.enc.check_enc_status(unsafe {
            JxlEncoderAddImageFrame(
                settings,
                &frame.pixel_format(),
                frame.data.as_ptr().cast(),
                std::mem::size_of_val(frame.data),
            )
        })?;
        self.last_frame = LastFrame::Queued;
        Ok(())
    }

    /// Keep the data needed to reconstruct the JPEG frame bit-exactly.
    /// Call it before adding any frame.
    ///
    /// # Errors
    /// Return [`EncodeError`] if output was already taken
    pub fn store_jpeg_metadata(&mut self) -> Result<(), EncodeError> {
        // SAFETY: `self.enc.enc` is valid until drop
        self.enc
            .check_enc_status(unsafe { JxlEncoderStoreJPEGMetadata(self.enc.enc, true.into()) })
    }

    /// Add a frame from JPEG data, which is recompressed losslessly
    ///
    /// # Errors
    /// Return [`EncodeError`] if the JPEG data is invalid or not supported
    pub fn add_jpeg_frame(&mut self, data: &[u8]) -> Result<(), EncodeError> {
        let settings = self.create_frame_settings(&self.frame_settings())?;
        // SAFETY: `settings` is valid and the size matches `data`
        self.enc.check_enc_status(unsafe {
            JxlEncoderAddJPEGFrame(settings, data.as_ptr().cast(), data.len())
        })?;
        self.last_frame = LastFrame::Queued;
        Ok(())
    }

    /// Encode everything added so far and return the output since the last call.
    ///
    /// The frames are written as non-final frames, so only call it when more frames follow.
    ///
    /// # Errors
    /// Return [`EncodeError`] if the encoder fails
    pub fn take_output(&mut self) -> Result<Vec<u8>, EncodeError> {
        self.process_output()?;
        if self.last_frame == LastFrame::Queued {
            self.last_frame = LastFrame::Taken;
        }
        Ok(std::mem::take(&mut self.output))
    }

    /// Finish the image and return the output since the last
    /// [`take_output`](Self::take_output).
    ///
    /// # Errors
    /// Return [`EncodeError`] if the encoder fails, or [`EncodeError::InvalidState`]
    /// if the last frame was already taken by `take_output`
    pub fn finish(mut self) -> Result<Vec<u8>, EncodeError> {
        if self.last_frame == LastFrame::Taken {
            return Err(EncodeError::InvalidState(
                "the last frame was written as a non-final frame",
            ));
        }
        // SAFETY: `self.enc.enc` is valid until drop
        unsafe { JxlEncoderCloseInput(self.enc.enc) };
        self.process_output()?;
        self.enc.boxes.clear();
        self.output.shrink_to_fit();
        Ok(std::mem::take(&mut self.output))
    }

    fn process_output(&mut self) -> Result<(), EncodeError> {
        let Self { enc, output, .. } = self;
        loop {
            let start = output.len();
            output.resize(start + enc.init_buffer_size.max(start), 0);
            let mut next_out = output[start..].as_mut_ptr();
            let mut avail_out = output.len() - start;
            // SAFETY: `next_out` and `avail_out` describe the unused tail of `output`
            let status =
                unsafe { JxlEncoderProcessOutput(enc.enc, &raw mut next_out, &raw mut avail_out) };
            output.truncate(output.len() - avail_out);
            if status != JxlEncoderStatus::NeedMoreOutput {
                return enc.check_enc_status(status);
            }
        }
    }
}

impl Drop for Session<'_, '_, '_> {
    fn drop(&mut self) {
        // SAFETY: `self.enc.enc` is valid until drop
        unsafe { JxlEncoderReset(self.enc.enc) };
    }
}
