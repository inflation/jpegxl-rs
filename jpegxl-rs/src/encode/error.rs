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

use eros::{
    type_set::{Contains, GroupNarrow, SupersetOf},
    ErrorUnion, TypeSet,
};
use jpegxl_sys::encoder::encode::JxlEncoderError;
use thiserror::Error;

use crate::errors::GenericError;

/// `libjxl` failed to create an encoder, usually because the memory manager is out of memory
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Cannot create an encoder")]
pub struct CannotCreateEncoder;

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

/// A frame name contains a NUL byte
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Invalid frame name")]
pub struct InvalidFrameName(#[from] pub std::ffi::NulError);

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

    use crate::{encode::JxlEncoder, errors::Failure, tests::failure};

    use super::*;

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn encode_invalid_data() -> TestResult {
        let mut encoder = JxlEncoder::builder().has_alpha(true).build()?;

        println!("{}", encoder.encode::<u8>(&[], 0, 0).err().unwrap());

        // One-shot functions report misuse as a `Failure` holding the libjxl error
        for result in [
            encoder.encode::<u8>(&[], 0, 0),
            encoder.encode::<f32>(&[1.0, 1.0, 1.0, 0.5], 1, 1),
        ] {
            let failure = failure::<Failure, _, _>(&result).expect("a failure");
            assert_eq!(failure.downcast_ref::<ApiUsage>(), Some(&ApiUsage));
        }

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

        // Every code maps to its own type
        for code in [
            JxlEncoderError::Generic,
            JxlEncoderError::OutOfMemory,
            JxlEncoderError::Jbrd,
            JxlEncoderError::BadInput,
            JxlEncoderError::NotSupported,
            JxlEncoderError::ApiUsage,
        ] {
            let error: ErrorUnion<(
                GenericError,
                OutOfMemory,
                Jbrd,
                BadInput,
                NotSupported,
                ApiUsage,
                UnspecifiedError,
            )> = enc_error(code);
            assert!(error.narrow::<UnspecifiedError, _>().is_err());
        }
        let x = UnspecifiedError;
        println!("{x}, {x:?}");
    }
}
