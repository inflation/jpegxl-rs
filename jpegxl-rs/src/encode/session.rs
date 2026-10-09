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

use std::{ffi::CString, mem::MaybeUninit, ptr::null};

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::{
    common::types::{JxlBitDepth, JxlBitDepthType},
    encoder::encode::*,
};

use super::{
    AddFrameErrors, AddJpegFrameErrors, ApiUsage, BadInput, ColorEncoding, EncoderFrame,
    FrameSettings, ImageInfo, InvalidFrameName, Jbrd, JxlEncoder, Metadata, NotSupported,
    OutOfMemory, UnspecifiedError,
};
use eros::{IntoUnion, ReshapeUnion};

use crate::{
    common::PixelType,
    errors::{GenericError, InvalidState},
};

/// An encoding session that is driven by the caller.
///
/// Add frames and boxes, then [`finish`](Self::finish) to get the encoded image.
///
/// ```
/// # use jpegxl_rs::{encoder_builder, encode::{EncoderFrame, ImageInfo}};
/// # fn main() -> jpegxl_rs::eros::Result<()> {
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
    ) -> eros::Result<Self, (ApiUsage, OutOfMemory, GenericError, UnspecifiedError)> {
        let session = Self {
            enc,
            output: Vec::new(),
            last_frame: LastFrame::None,
        };
        session.setup(info)?;
        for (t, data, compress) in &session.enc.boxes {
            session.add_box(*t, data, *compress).widen()?;
        }
        Ok(session)
    }

    fn setup(
        &self,
        info: Option<&ImageInfo>,
    ) -> eros::Result<(), (ApiUsage, OutOfMemory, GenericError, UnspecifiedError)> {
        let enc = &*self.enc;
        if let Some(runner) = enc.parallel_runner {
            // SAFETY: `enc.enc` is valid until drop and the runner outlives it
            enc.check_enc_status::<(ApiUsage, OutOfMemory, UnspecifiedError), _, _>(unsafe {
                JxlEncoderSetParallelRunner(enc.enc, runner.runner(), runner.as_opaque_ptr())
            })
            .widen()?;
        }
        // SAFETY: `enc.enc` is valid until drop
        enc.check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
            JxlEncoderUseContainer(enc.enc, enc.use_container.into())
        })
        .widen()?;
        if enc.use_box {
            // SAFETY: `enc.enc` is valid until drop
            enc.check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
                JxlEncoderUseBoxes(enc.enc)
            })
            .widen()?;
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
        if let Some(animation) = info.animation {
            basic_info.have_animation = true.into();
            basic_info.animation.tps_numerator = animation.tps_numerator;
            basic_info.animation.tps_denominator = animation.tps_denominator;
            basic_info.animation.num_loops = animation.num_loops;
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
        enc.check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
            JxlEncoderSetBasicInfo(enc.enc, &raw const basic_info)
        })
        .widen()?;
        if let Some(color_encoding) = &enc.color_encoding {
            // SAFETY: `enc.enc` is valid until drop
            enc.check_enc_status::<(ApiUsage, GenericError, UnspecifiedError), _, _>(unsafe {
                JxlEncoderSetColorEncoding(enc.enc, &color_encoding.into())
            })
            .widen()?;
        }
        Ok(())
    }

    /// Add a metadata box
    ///
    /// # Errors
    /// Return [`ApiUsage`] if boxes cannot be added anymore, [`OutOfMemory`], or
    /// [`UnspecifiedError`]
    pub fn add_metadata(
        &mut self,
        metadata: &Metadata,
        compress: bool,
    ) -> eros::Result<(), (ApiUsage, OutOfMemory, UnspecifiedError)> {
        let (t, data) = metadata.parts();
        self.add_box(t, data, compress)
    }

    fn add_box(
        &self,
        t: [u8; 4],
        data: &[u8],
        compress: bool,
    ) -> eros::Result<(), (ApiUsage, OutOfMemory, UnspecifiedError)> {
        let enc = &*self.enc;
        // SAFETY: `enc.enc` is valid until drop
        enc.check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
            JxlEncoderUseBoxes(enc.enc)
        })
        .widen()?;
        // SAFETY: `enc.enc` is valid until drop
        enc.check_enc_status::<(ApiUsage, OutOfMemory, UnspecifiedError), _, _>(unsafe {
            JxlEncoderAddBox(
                enc.enc,
                &Metadata::box_type(t),
                data.as_ptr().cast(),
                data.len(),
                compress.into(),
            )
        })
        .widen()
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
    ) -> eros::Result<
        *mut JxlEncoderFrameSettings,
        (OutOfMemory, ApiUsage, NotSupported, UnspecifiedError),
    > {
        let enc = &*self.enc;
        // SAFETY: `enc.enc` is valid until drop
        let ptr = unsafe { JxlEncoderFrameSettingsCreate(enc.enc, null()) };
        if ptr.is_null() {
            return Err(OutOfMemory).union();
        }

        let set = |id, value| {
            // SAFETY: `ptr` is valid until the encoder is reset
            enc.check_enc_status::<(ApiUsage, NotSupported, UnspecifiedError), _, _>(unsafe {
                JxlEncoderFrameSettingsSetOption(ptr, id, value)
            })
            .widen()
        };
        if let Some(lossless) = settings.lossless {
            // SAFETY: `ptr` is valid until the encoder is reset
            enc.check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
                JxlEncoderSetFrameLossless(ptr, lossless.into())
            })
            .widen()?;
        }
        set(JxlEncoderFrameSettingId::Effort, settings.speed as _)?;
        // SAFETY: `ptr` is valid until the encoder is reset
        enc.check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
            JxlEncoderSetFrameDistance(ptr, settings.quality)
        })
        .widen()?;
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
    /// Return [`ApiUsage`] or [`NotSupported`] if the frame or its settings are invalid,
    /// [`OutOfMemory`], [`GenericError`], [`UnspecifiedError`], or [`InvalidFrameName`] if its name
    /// contains a NUL byte
    pub fn add_frame<T: PixelType>(
        &mut self,
        frame: &EncoderFrame<T>,
    ) -> eros::Result<(), AddFrameErrors> {
        let settings = match frame.settings {
            Some(settings) => self.create_frame_settings(settings).widen()?,
            None => self.create_frame_settings(&self.frame_settings()).widen()?,
        };
        let enc = &*self.enc;
        if let Some(duration) = frame.duration {
            // SAFETY: `JxlEncoderInitFrameHeader` initializes `header`
            let mut header = unsafe {
                let mut header = MaybeUninit::uninit();
                JxlEncoderInitFrameHeader(header.as_mut_ptr());
                header.assume_init()
            };
            header.duration = duration;
            // SAFETY: `settings` is valid until the encoder is reset
            enc.check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
                JxlEncoderSetFrameHeader(settings, &raw const header)
            })
            .widen()?;
        }
        if let Some(name) = frame.name {
            let name = CString::new(name).map_err(InvalidFrameName).union()?;
            // SAFETY: `settings` is valid until the encoder is reset
            enc.check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
                JxlEncoderSetFrameName(settings, name.as_ptr().cast())
            })
            .widen()?;
        }
        if frame.bit_depth_from_image {
            let bit_depth = JxlBitDepth {
                r#type: JxlBitDepthType::FromCodestream,
                bits_per_sample: 0,
                exponent_bits_per_sample: 0,
            };
            // SAFETY: `settings` is valid until the encoder is reset
            enc.check_enc_status::<(UnspecifiedError,), _, _>(unsafe {
                JxlEncoderSetFrameBitDepth(settings, &raw const bit_depth)
            })
            .widen()?;
        }
        // SAFETY: `settings` is valid and the size matches `frame.data`
        let status = unsafe {
            JxlEncoderAddImageFrame(
                settings,
                &frame.pixel_format(),
                frame.data.as_ptr().cast(),
                std::mem::size_of_val(frame.data),
            )
        };
        enc.check_enc_status::<(ApiUsage, GenericError, OutOfMemory, UnspecifiedError), _, _>(
            status,
        )
        .widen()?;
        self.last_frame = LastFrame::Queued;
        Ok(())
    }

    /// Keep the data needed to reconstruct the JPEG frame bit-exactly.
    /// Call it before adding any frame.
    ///
    /// # Errors
    /// Return [`ApiUsage`] if output was already taken
    pub fn store_jpeg_metadata(&mut self) -> eros::Result<(), (ApiUsage, UnspecifiedError)> {
        // SAFETY: `self.enc.enc` is valid until drop
        self.enc
            .check_enc_status::<(ApiUsage, UnspecifiedError), _, _>(unsafe {
                JxlEncoderStoreJPEGMetadata(self.enc.enc, true.into())
            })
            .widen()
    }

    /// Add a frame from JPEG data, which is recompressed losslessly
    ///
    /// # Errors
    /// Return [`BadInput`] if the JPEG data is invalid, [`NotSupported`] or [`Jbrd`] if it
    /// cannot be recompressed, [`ApiUsage`], [`OutOfMemory`], [`GenericError`], or
    /// [`UnspecifiedError`]
    pub fn add_jpeg_frame(&mut self, data: &[u8]) -> eros::Result<(), AddJpegFrameErrors> {
        let settings = self.create_frame_settings(&self.frame_settings()).widen()?;
        // SAFETY: `settings` is valid and the size matches `data`
        let status = unsafe { JxlEncoderAddJPEGFrame(settings, data.as_ptr().cast(), data.len()) };
        self.enc
            .check_enc_status::<(
                GenericError,
                OutOfMemory,
                Jbrd,
                BadInput,
                NotSupported,
                ApiUsage,
                UnspecifiedError,
            ), _, _>(status)
            .widen()?;
        self.last_frame = LastFrame::Queued;
        Ok(())
    }

    /// Encode everything added so far and return the output since the last call.
    ///
    /// The frames are written as non-final frames, so only call it when more frames follow.
    ///
    /// # Errors
    /// Return [`ApiUsage`], [`GenericError`] or [`UnspecifiedError`] if the encoder fails
    pub fn take_output(
        &mut self,
    ) -> eros::Result<Vec<u8>, (ApiUsage, GenericError, UnspecifiedError)> {
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
    /// Return [`ApiUsage`], [`GenericError`] or [`UnspecifiedError`] if the encoder fails,
    /// or [`InvalidState`]
    /// if the last frame was already taken by `take_output`
    pub fn finish(
        mut self,
    ) -> eros::Result<Vec<u8>, (InvalidState, ApiUsage, GenericError, UnspecifiedError)> {
        if self.last_frame == LastFrame::Taken {
            return Err(InvalidState(
                "the last frame was written as a non-final frame",
            ))
            .union();
        }
        // SAFETY: `self.enc.enc` is valid until drop
        unsafe { JxlEncoderCloseInput(self.enc.enc) };
        self.process_output().widen()?;
        self.enc.boxes.clear();
        self.output.shrink_to_fit();
        Ok(std::mem::take(&mut self.output))
    }

    fn process_output(&mut self) -> eros::Result<(), (ApiUsage, GenericError, UnspecifiedError)> {
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
                return enc
                    .check_enc_status::<(ApiUsage, GenericError, UnspecifiedError), _, _>(status)
                    .widen();
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
