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

//! Decoder and encoder errors
//!
//! Every failure is its own small type. Functions return an [`eros::ErrorUnion`]
//! of exactly the failures they can produce, e.g.
//! `eros::Result<_, (InvalidState, DecoderStatus)>`, so callers only handle what can
//! actually happen. A union converts into a wider one with
//! [`widen`](eros::ErrorUnion::widen), and a single failure is picked out with
//! [`narrow`](eros::ErrorUnion::narrow) or tested with
//! [`is_inner`](eros::ErrorUnion::is_inner).

use eros::{type_set::Contains, ErrorUnion, TypeSet};
use thiserror::Error;

use jpegxl_sys::{decode::JxlDecoderStatus, encoder::encode::JxlEncoderError};

/// `libjxl` failed to create a decoder, usually because the memory manager is out of memory
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Cannot create a decoder")]
pub struct CannotCreateDecoder;

/// `libjxl` failed to create an encoder, usually because the memory manager is out of memory
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Cannot create an encoder")]
pub struct CannotCreateEncoder;

/// A `libjxl` decoder call returned a non-success status
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("{}", decoder_status_message(*.0))]
pub struct DecoderStatus(pub JxlDecoderStatus);

fn decoder_status_message(status: JxlDecoderStatus) -> String {
    match status {
        JxlDecoderStatus::Error => "Generic Error. Please build `libjxl` from source (using \
            `vendored` feature) in debug mode to get more information. Check `stderr` for any \
            internal error messages."
            .to_owned(),
        s => format!("Unknown status: `{s:?}`"),
    }
}

/// A `libjxl` encoder call failed
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("{}", encoder_error_message(*.0))]
pub struct EncoderStatus(pub JxlEncoderError);

fn encoder_error_message(error: JxlEncoderError) -> &'static str {
    match error {
        JxlEncoderError::OK => "No error",
        JxlEncoderError::Generic => {
            "Generic Error. Please build `libjxl` from source (using `vendored` feature) in \
            debug mode to get more information. Check `stderr` for any internal error messages."
        }
        JxlEncoderError::OutOfMemory => "Out of memory",
        JxlEncoderError::Jbrd => "JPEG bitstream reconstruction data could not be represented",
        JxlEncoderError::BadInput => "Input is invalid",
        JxlEncoderError::NotSupported => "Encoder does not support it (yet)",
        JxlEncoderError::ApiUsage => "The encoder API is used in an incorrect way",
    }
}

/// The input does not contain a valid codestream or container
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("The input does not contain a valid codestream or container")]
pub struct InvalidInput;

/// The input ended before the image was complete
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("The input ended before the image was complete")]
pub struct IncompleteInput;

/// The image uses a pixel bit width without a matching Rust type
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Unsupported Pixel bit width: {0}")]
pub struct UnsupportedBitWidth(pub u32);

/// A frame name contains a NUL byte
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Invalid frame name")]
pub struct InvalidFrameName(#[from] pub std::ffi::NulError);

/// Internal error, usually invalid usages of the `libjxl` library
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Internal error, please file an issue: {0}")]
pub struct InternalError(pub &'static str);

/// A session method is called at the wrong time
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Invalid session state: {0}")]
pub struct InvalidState(pub &'static str);

/// Everything [`Session::process`](crate::decode::Session::process) can fail with
pub type ProcessError = (DecoderStatus, IncompleteInput, InternalError);

/// Everything a one-shot decode, e.g. [`JxlDecoder::decode`](crate::decode::JxlDecoder::decode),
/// can fail with
pub type DecodeError = (
    InvalidInput,
    IncompleteInput,
    DecoderStatus,
    UnsupportedBitWidth,
    InvalidState,
    InternalError,
);

/// Everything a one-shot encode, e.g. [`JxlEncoder::encode`](crate::encode::JxlEncoder::encode),
/// can fail with
pub type EncodeError = (EncoderStatus, InvalidFrameName, InvalidState);

/// Error mapping from underlying C const to [`DecoderStatus`], in any union that contains it
pub(crate) fn check_dec_status<S, I>(status: JxlDecoderStatus) -> eros::Result<(), S>
where
    S: TypeSet,
    S::Variants: Contains<DecoderStatus, I>,
{
    match status {
        JxlDecoderStatus::Success => Ok(()),
        _ => Err(ErrorUnion::new(DecoderStatus(status))),
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use crate::{encode::JxlEncoder, tests::failure};

    use super::*;

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn decode_invalid_data() -> TestResult {
        let decoder = crate::decoder_builder().build()?;
        assert!(failure::<InvalidInput, _, _>(&decoder.decode_with::<u8>(&[])).is_some());
        assert!(failure::<InvalidInput, _, _>(&decoder.decode_with::<u8>(&[0; 64])).is_some());
        assert!(failure::<IncompleteInput, _, _>(
            &decoder.decode(&crate::tests::SAMPLE_JXL[..100])
        )
        .is_some());

        let status: eros::Result<(), (DecoderStatus,)> = check_dec_status(JxlDecoderStatus::Error);
        assert_eq!(*status.unwrap_err(), DecoderStatus(JxlDecoderStatus::Error));

        let status: eros::Result<(), (DecoderStatus,)> =
            check_dec_status(JxlDecoderStatus::BasicInfo);
        println!("{x}, {x:?}", x = status.unwrap_err());

        Ok(())
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn encode_invalid_data() -> TestResult {
        let mut encoder = JxlEncoder::builder().has_alpha(true).build()?;

        println!("{}", encoder.encode::<u8>(&[], 0, 0).err().unwrap());

        let api_usage = Some(&EncoderStatus(JxlEncoderError::ApiUsage));
        assert_eq!(failure(&encoder.encode::<u8>(&[], 0, 0)), api_usage);
        assert_eq!(
            failure(&encoder.encode::<f32>(&[1.0, 1.0, 1.0, 0.5], 1, 1)),
            api_usage
        );

        println!("{x}, {x:?}", x = EncoderStatus(JxlEncoderError::OK));

        Ok(())
    }
}
