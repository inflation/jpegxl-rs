use std::{ffi::CString, mem::MaybeUninit, sync::Arc};

use jpegxl_sys::{
    color::color_encoding::JxlColorEncoding, common::types::JxlPixelFormat, decode as d,
};

use super::{BasicInfo, ColorProfileTarget, Event, JxlDecoder, Pixels};
use crate::{decode::parse_events, errors::check_dec_status, DecodeError};

/// Represents the state of the session.
pub enum State {
    /// Waiting for the next event.
    Continue,
    /// Basic information such as image dimensions and extra channels.
    /// This event occurs max once per image.
    BasicInfo(Arc<BasicInfo>),
    /// ICC color profile .
    IccProfile(Vec<u8>),
    /// Color profile.
    ColorProfile(Box<JxlColorEncoding>),
    /// Preview image. Dimensions can be accessed from [`BasicInfo::preview`]
    PreviewImage(Pixels),
    /// Begining of a frame
    Frame,
    /// JPEG reconstruction.
    JpegReconstruction(Vec<u8>),
}

#[derive(Debug, Default)]
pub(crate) struct Config {
    pub color_profile: Option<ColorEncodingConfig>,
    pub preview: Option<JxlPixelFormat>,
    pub frame: Option<usize>,
    pub jpeg_reconstruction: Option<usize>,
}

/// Configuration for color encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEncodingConfig {
    target: ColorProfileTarget,
    icc_profile: bool,
}

/// Represents a session for decoding JPEG XL images.
pub struct Session<'dec, 'pr, 'mm> {
    dec: &'dec mut JxlDecoder<'pr, 'mm>,
    basic_info: Option<Arc<BasicInfo>>,
    jpeg_buffer: Vec<u8>,
    config: Config,
    state: State,
}

impl<'dec, 'pr, 'mm> Session<'dec, 'pr, 'mm> {
    pub(crate) fn new<I>(
        dec: &'dec mut JxlDecoder<'pr, 'mm>,
        registered_events: I,
    ) -> Result<Self, DecodeError>
    where
        I: IntoIterator<Item = Event>,
    {
        use jpegxl_sys::decode::{JxlDecoderSetParallelRunner, JxlDecoderSubscribeEvents};

        if let Some(runner) = dec.parallel_runner {
            check_dec_status(unsafe {
                JxlDecoderSetParallelRunner(dec.ptr, runner.runner(), runner.as_opaque_ptr())
            })?;
        }

        let (flags, config) = parse_events(registered_events);
        check_dec_status(unsafe { JxlDecoderSubscribeEvents(dec.ptr, flags) })?;

        macro_rules! set_value {
            ( $( ($name:ident, $fn:ident) $(,)? )* ) => {
                $(
                    if let Some(val) = dec.$name {
                        check_dec_status(unsafe { jpegxl_sys::decode::$fn(dec.ptr, val.into()) })?;
                    }
                )*
            };
        }

        set_value! {
            (skip_reorientation, JxlDecoderSetKeepOrientation),
            (unpremul_alpha, JxlDecoderSetUnpremultiplyAlpha),
            (render_spotcolors, JxlDecoderSetRenderSpotcolors),
            (coalescing, JxlDecoderSetCoalescing),
            (
                desired_intensity_target,
                JxlDecoderSetDesiredIntensityTarget
            )
        }

        Ok(Self {
            dec,
            basic_info: None,
            jpeg_buffer: Vec::new(),
            config,
            state: State::Continue,
        })
    }

    fn step(&mut self, status: d::JxlDecoderStatus) -> Result<State, DecodeError> {
        use jpegxl_sys::decode::JxlDecoderStatus as s;

        match status {
            s::Success => panic!("Unexpected success status"),
            s::Error => Err(DecodeError::GenericError),
            s::BasicInfo => self.get_basic_info(),
            s::ColorEncoding => self.get_color_profile(),
            s::PreviewImage => self.get_preview_image(),
            s::Frame => self.get_frame(),
            s::JPEGReconstruction => {
                let Some(size) = self.config.jpeg_reconstruction else {
                    return Err(DecodeError::InternalError(
                        "Subscribe to JPEG reconstruction event but without a config!",
                    ));
                };

                self.jpeg_buffer.resize(size, 0);

                check_dec_status(unsafe {
                    d::JxlDecoderSetJPEGBuffer(
                        self.dec.ptr,
                        self.jpeg_buffer.as_mut_ptr(),
                        self.jpeg_buffer.len(),
                    )
                })?;

                Ok(State::Continue)
            }
            s::JPEGNeedMoreOutput => {
                let remaining = unsafe { d::JxlDecoderReleaseJPEGBuffer(self.dec.ptr) };

                self.jpeg_buffer
                    .resize(self.jpeg_buffer.len() + remaining, 0);

                check_dec_status(unsafe {
                    d::JxlDecoderSetJPEGBuffer(
                        self.dec.ptr,
                        self.jpeg_buffer.as_mut_ptr(),
                        self.jpeg_buffer.len(),
                    )
                })?;

                Ok(State::Continue)
            }
            _ => unimplemented!(),
        }
    }

    fn get_basic_info(&mut self) -> Result<State, DecodeError> {
        let mut info = MaybeUninit::uninit();
        check_dec_status(unsafe { d::JxlDecoderGetBasicInfo(self.dec.ptr, info.as_mut_ptr()) })?;

        if let Some(pr) = self.dec.parallel_runner {
            pr.callback_basic_info(unsafe { &*info.as_ptr() });
        }

        unsafe {
            self.basic_info = Some(Arc::new(info.assume_init()));
            Ok(State::BasicInfo(
                self.basic_info.as_ref().unwrap_unchecked().clone(),
            ))
        }
    }

    fn get_color_profile(&mut self) -> Result<State, DecodeError> {
        let Some(config) = self.config.color_profile else {
            return Err(DecodeError::InternalError(
                "Subscribe to color encoding event but without a color profile config!",
            ));
        };

        if config.icc_profile {
            let mut icc_size = 0;
            let mut icc_profile = Vec::new();

            check_dec_status(unsafe {
                d::JxlDecoderGetICCProfileSize(self.dec.ptr, config.target, &mut icc_size)
            })?;
            icc_profile.resize(icc_size, 0);

            check_dec_status(unsafe {
                d::JxlDecoderGetColorAsICCProfile(
                    self.dec.ptr,
                    config.target,
                    icc_profile.as_mut_ptr(),
                    icc_size,
                )
            })?;

            Ok(State::IccProfile(icc_profile))
        } else {
            let mut color_encoding = MaybeUninit::uninit();

            check_dec_status(unsafe {
                d::JxlDecoderGetColorAsEncodedProfile(
                    self.dec.ptr,
                    config.target,
                    color_encoding.as_mut_ptr(),
                )
            })?;
            Ok(State::ColorProfile(Box::new(unsafe {
                color_encoding.assume_init()
            })))
        }
    }

    fn get_preview_image(&mut self) -> Result<State, DecodeError> {
        let Some(pixel_format) = self.config.preview else {
            return Err(DecodeError::InternalError(
                "Subscribe to preview image event but without a pixel format!",
            ));
        };

        let mut size = 0;

        check_dec_status(unsafe {
            d::JxlDecoderPreviewOutBufferSize(self.dec.ptr, &pixel_format, &mut size)
        })?;

        let mut buffer = vec![0; size];
        check_dec_status(unsafe {
            d::JxlDecoderSetPreviewOutBuffer(
                self.dec.ptr,
                &pixel_format,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
            )
        })?;

        Ok(State::PreviewImage(Pixels::new(buffer, &pixel_format)))
    }

    fn get_frame(&mut self) -> Result<State, DecodeError> {
        let mut header = MaybeUninit::uninit();
        check_dec_status(unsafe {
            d::JxlDecoderGetFrameHeader(self.dec.ptr, header.as_mut_ptr())
        })?;
        let header = unsafe { header.assume_init() };

        let mut buffer = vec![0; header.name_length as usize + 1];
        check_dec_status(unsafe {
            d::JxlDecoderGetFrameName(self.dec.ptr, buffer.as_mut_ptr().cast(), buffer.len())
        })?;
        let name = CString::from_vec_with_nul(buffer)
            .map_err(|_| DecodeError::InternalError("Invalid frame name"))?;

        Ok(State::Frame)
    }
}

impl<'dec, 'pr, 'mm> Iterator for Session<'dec, 'pr, 'mm> {
    type Item = Result<State, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        use jpegxl_sys::decode::{JxlDecoderProcessInput, JxlDecoderStatus as s};

        let status = unsafe { JxlDecoderProcessInput(self.dec.ptr) };

        match status {
            s::Success => None,
            status => Some(self.step(status)),
        }
    }
}

impl Drop for Session<'_, '_, '_> {
    fn drop(&mut self) {
        unsafe { jpegxl_sys::decode::JxlDecoderReset(self.dec.ptr) }
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use crate::decoder_builder;

    use super::*;

    #[test]
    fn test_session() -> TestResult {
        let mut decoder = decoder_builder().build()?;
        let session = Session::new(&mut decoder, [Event::BasicInfo])?;
        drop(session);

        Ok(())
    }
}
