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
//! `eros::Result<_, (InvalidState, GenericError)>`, so callers only handle what can
//! actually happen. The `libjxl` failures of each call are taken from the `libjxl`
//! sources. A union converts into a wider one with
//! [`widen`](eros::ErrorUnion::widen), and a single failure is picked out with
//! [`narrow`](eros::ErrorUnion::narrow).

use eros::{
    type_set::{Contains, GroupNarrow, SupersetOf},
    ErrorUnion, TypeSet,
};
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

/// `libjxl` reported a generic error (`JXL_DEC_ERROR` or `JXL_ENC_ERR_GENERIC`)
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error(
    "Generic Error. Please build `libjxl` from source (using `vendored` feature) \
    in debug mode to get more information. Check `stderr` for any internal error messages."
)]
pub struct GenericError;

/// A `libjxl` decoder call returned a status other than success or error,
/// e.g. `NeedMoreInput` when the requested information is not decoded yet
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Unexpected decoder status: `{0:?}`")]
pub struct UnexpectedStatus(pub JxlDecoderStatus);

/// The encoder ran out of memory (`JXL_ENC_ERR_OOM`)
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Out of memory")]
pub struct OutOfMemory;

/// JPEG bitstream reconstruction data could not be represented, e.g. too much tail data
/// (`JXL_ENC_ERR_JBRD`)
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("JPEG bitstream reconstruction data could not be represented")]
pub struct Jbrd;

/// Input is invalid, e.g. a corrupt JPEG file or ICC profile (`JXL_ENC_ERR_BAD_INPUT`)
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Input is invalid")]
pub struct BadInput;

/// The encoder does not support it (yet) (`JXL_ENC_ERR_NOT_SUPPORTED`). Since libjxl v0.12,
/// also returned when parsing a JPEG fails due to features not supported for recompression
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Encoder does not support it (yet)")]
pub struct NotSupported;

/// The encoder API is used in an incorrect way (`JXL_ENC_ERR_API_USAGE`).
/// A debug build of libjxl outputs a specific error message
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("The encoder API is used in an incorrect way")]
pub struct ApiUsage;

/// The encoder failed without an error code for this call
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error(
    "The encoder failed without an error code. Please build `libjxl` from source (using \
    `vendored` feature) in debug mode to get more information."
)]
pub struct UnspecifiedError;

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

/// Map a decoder status to the failures `S` of the call that returned it.
///
/// A status outside `S` contradicts the `libjxl` sources, and becomes a [`GenericError`]
pub(crate) fn check_dec_status<S, I, J>(status: JxlDecoderStatus) -> eros::Result<(), S>
where
    S: TypeSet,
    S::Variants: Contains<GenericError, J>,
    <(GenericError, UnexpectedStatus) as TypeSet>::Variants: SupersetOf<S::Variants, I>,
{
    let error: ErrorUnion<(GenericError, UnexpectedStatus)> = match status {
        JxlDecoderStatus::Success => return Ok(()),
        JxlDecoderStatus::Error => ErrorUnion::new(GenericError),
        s => ErrorUnion::new(UnexpectedStatus(s)),
    };
    Err(error
        .narrow::<S, GroupNarrow<I>>()
        .unwrap_or_else(|_| ErrorUnion::new(GenericError)))
}

/// Map an encoder error code to the failures `S` of the call that failed.
///
/// `libjxl` keeps the code of an earlier failure, and some failures set none, so a code
/// outside `S` becomes an [`UnspecifiedError`]
pub(crate) fn enc_error<S, I, J>(error: JxlEncoderError) -> ErrorUnion<S>
where
    S: TypeSet,
    S::Variants: Contains<UnspecifiedError, J>,
    <(
        GenericError,
        OutOfMemory,
        Jbrd,
        BadInput,
        NotSupported,
        ApiUsage,
        UnspecifiedError,
    ) as TypeSet>::Variants: SupersetOf<S::Variants, I>,
{
    let error: ErrorUnion<(
        GenericError,
        OutOfMemory,
        Jbrd,
        BadInput,
        NotSupported,
        ApiUsage,
        UnspecifiedError,
    )> = match error {
        JxlEncoderError::OK => ErrorUnion::new(UnspecifiedError),
        JxlEncoderError::Generic => ErrorUnion::new(GenericError),
        JxlEncoderError::OutOfMemory => ErrorUnion::new(OutOfMemory),
        JxlEncoderError::Jbrd => ErrorUnion::new(Jbrd),
        JxlEncoderError::BadInput => ErrorUnion::new(BadInput),
        JxlEncoderError::NotSupported => ErrorUnion::new(NotSupported),
        JxlEncoderError::ApiUsage => ErrorUnion::new(ApiUsage),
    };
    error
        .narrow::<S, GroupNarrow<I>>()
        .unwrap_or_else(|_| ErrorUnion::new(UnspecifiedError))
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

        let status: eros::Result<(), (GenericError, UnexpectedStatus)> =
            check_dec_status(JxlDecoderStatus::Error);
        assert!(failure::<GenericError, _, _>(&status).is_some());

        let status: eros::Result<(), (GenericError, UnexpectedStatus)> =
            check_dec_status(JxlDecoderStatus::NeedMoreInput);
        assert_eq!(
            failure(&status),
            Some(&UnexpectedStatus(JxlDecoderStatus::NeedMoreInput))
        );
        println!("{x}, {x:?}", x = status.unwrap_err());

        // A status the call never returns folds into the generic error
        let status: eros::Result<(), (GenericError,)> =
            check_dec_status(JxlDecoderStatus::NeedMoreInput);
        assert_eq!(*status.unwrap_err(), GenericError);

        Ok(())
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn encode_invalid_data() -> TestResult {
        let mut encoder = JxlEncoder::builder().has_alpha(true).build()?;

        println!("{}", encoder.encode::<u8>(&[], 0, 0).err().unwrap());

        assert!(failure::<ApiUsage, _, _>(&encoder.encode::<u8>(&[], 0, 0)).is_some());
        assert!(
            failure::<ApiUsage, _, _>(&encoder.encode::<f32>(&[1.0, 1.0, 1.0, 0.5], 1, 1))
                .is_some()
        );

        Ok(())
    }

    #[test]
    fn encoder_codes_outside_the_call_are_unspecified() {
        let error: ErrorUnion<(ApiUsage, UnspecifiedError)> = enc_error(JxlEncoderError::ApiUsage);
        assert!(error.narrow::<ApiUsage, _>().is_ok());

        // A stale code from an earlier call, or none at all
        for code in [JxlEncoderError::OutOfMemory, JxlEncoderError::OK] {
            let error: ErrorUnion<(ApiUsage, UnspecifiedError)> = enc_error(code);
            assert_eq!(
                error.narrow::<UnspecifiedError, _>().ok(),
                Some(UnspecifiedError)
            );
        }
        let x = UnspecifiedError;
        println!("{x}, {x:?}");
    }
}
