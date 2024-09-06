use std::{mem::MaybeUninit, sync::Arc};

use jpegxl_sys::{
    color_encoding::JxlColorEncoding,
    decode::{
        JxlDecoderGetColorAsEncodedProfile, JxlDecoderGetColorAsICCProfile,
        JxlDecoderGetICCProfileSize, JxlDecoderStatus,
    },
};

use super::{BasicInfo, ColorProfileTarget, Event, JxlDecoder};
use crate::{decode::parse_events, errors::check_dec_status, DecodeError};

/// Represents the state of the session.
pub enum State {
    /// Initial state of the session.
    Init,
    /// Basic information such as image dimensions and extra channels.
    /// This event occurs max once per image.
    BasicInfo(Arc<BasicInfo>),
    /// ICC color profile .
    IccProfile(Vec<u8>),
    ///
    ColorProfile(JxlColorEncoding),
}

#[derive(Debug, Default)]
pub(crate) struct Config {
    pub color_profile: Option<ColorEncodingConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEncodingConfig {
    target: ColorProfileTarget,
    icc_profile: bool,
}

/// Represents a session for decoding JPEG XL images.
pub struct Session<'dec, 'pr, 'mm> {
    dec: &'dec mut JxlDecoder<'pr, 'mm>,
    basic_info: Option<Arc<BasicInfo>>,
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
            config,
            state: State::Init,
        })
    }

    fn step(&mut self, status: JxlDecoderStatus) -> Result<State, DecodeError> {
        use jpegxl_sys::decode::{JxlDecoderGetBasicInfo, JxlDecoderStatus as s};

        match status {
            s::Success => panic!("Unexpected success status"),
            s::Error => Err(DecodeError::GenericError),
            s::BasicInfo => {
                let mut info = MaybeUninit::uninit();
                check_dec_status(unsafe {
                    JxlDecoderGetBasicInfo(self.dec.ptr, info.as_mut_ptr())
                })?;

                if let Some(pr) = self.dec.parallel_runner {
                    pr.callback_basic_info(unsafe { &*info.as_ptr() });
                }

                self.basic_info = Some(Arc::new(unsafe { info.assume_init() }));
                Ok(State::BasicInfo(unsafe {
                    self.basic_info.as_ref().unwrap_unchecked().clone()
                }))
            }
            s::ColorEncoding => {
                let Some(config) = self.config.color_profile else {
                    return Err(DecodeError::InvalidUsage(
                        "Subscribe to color encoding event but without a color profile",
                    ));
                };

                if config.icc_profile {
                    let mut icc_size = 0;
                    let mut icc_profile = Vec::new();

                    check_dec_status(unsafe {
                        JxlDecoderGetICCProfileSize(self.dec.ptr, config.target, &mut icc_size)
                    })?;
                    icc_profile.resize(icc_size, 0);

                    check_dec_status(unsafe {
                        JxlDecoderGetColorAsICCProfile(
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
                        JxlDecoderGetColorAsEncodedProfile(
                            self.dec.ptr,
                            config.target,
                            color_encoding.as_mut_ptr(),
                        )
                    })?;
                    Ok(State::ColorProfile(unsafe { color_encoding.assume_init() }))
                }
            }
            _ => unimplemented!(),
        }
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
