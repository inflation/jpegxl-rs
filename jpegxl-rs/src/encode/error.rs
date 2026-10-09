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

use jpegxl_sys::encoder::encode::JxlEncoderError;
use thiserror::Error;

/// `libjxl` failed to create an encoder, usually because the memory manager is out of memory
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Cannot create an encoder")]
pub struct CannotCreateEncoder;

/// A `libjxl` encoder call failed, with the error code it reported.
///
/// `libjxl` keeps the code until the encoder is reset, so after an ignored failure the code
/// may describe that earlier failure
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EncoderFailure {
    /// Generic error (`JXL_ENC_ERR_GENERIC`)
    #[error(
        "Generic Error. Please build `libjxl` from source (using `vendored` feature) in \
        debug mode to get more information. Check `stderr` for any internal error messages."
    )]
    Generic,
    /// Out of memory (`JXL_ENC_ERR_OOM`)
    #[error("Out of memory")]
    OutOfMemory,
    /// JPEG bitstream reconstruction data could not be represented, e.g. too much tail data
    /// (`JXL_ENC_ERR_JBRD`)
    #[error("JPEG bitstream reconstruction data could not be represented")]
    Jbrd,
    /// Input is invalid, e.g. a corrupt JPEG file or ICC profile (`JXL_ENC_ERR_BAD_INPUT`)
    #[error("Input is invalid")]
    BadInput,
    /// Not supported (yet) (`JXL_ENC_ERR_NOT_SUPPORTED`). Since libjxl v0.12, also returned
    /// when parsing a JPEG fails due to features not supported for recompression
    #[error("Encoder does not support it (yet)")]
    NotSupported,
    /// The encoder API is used in an incorrect way (`JXL_ENC_ERR_API_USAGE`).
    /// A debug build of libjxl outputs a specific error message
    #[error("The encoder API is used in an incorrect way")]
    ApiUsage,
    /// Failed without an error code
    #[error(
        "The encoder failed without an error code. Please build `libjxl` from source (using \
        `vendored` feature) in debug mode to get more information."
    )]
    Unspecified,
}

impl From<JxlEncoderError> for EncoderFailure {
    fn from(error: JxlEncoderError) -> Self {
        match error {
            JxlEncoderError::OK => Self::Unspecified,
            JxlEncoderError::Generic => Self::Generic,
            JxlEncoderError::OutOfMemory => Self::OutOfMemory,
            JxlEncoderError::Jbrd => Self::Jbrd,
            JxlEncoderError::BadInput => Self::BadInput,
            JxlEncoderError::NotSupported => Self::NotSupported,
            JxlEncoderError::ApiUsage => Self::ApiUsage,
        }
    }
}

/// A frame name contains a NUL byte
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Invalid frame name")]
pub struct InvalidFrameName(#[from] pub std::ffi::NulError);

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use crate::encode::JxlEncoder;

    use super::*;

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn encode_invalid_data() -> TestResult {
        let mut encoder = JxlEncoder::builder().has_alpha(true).build()?;

        println!("{}", encoder.encode::<u8>(&[], 0, 0).err().unwrap());

        for result in [
            encoder.encode::<u8>(&[], 0, 0),
            encoder.encode::<f32>(&[1.0, 1.0, 1.0, 0.5], 1, 1),
        ] {
            let failure = result.unwrap_err().narrow::<EncoderFailure, _>();
            assert_eq!(failure.ok(), Some(EncoderFailure::ApiUsage));
        }

        Ok(())
    }

    #[test]
    fn encoder_codes() {
        // No code, or the code of an earlier call
        assert_eq!(
            EncoderFailure::from(JxlEncoderError::OK),
            EncoderFailure::Unspecified
        );
        assert_eq!(
            EncoderFailure::from(JxlEncoderError::Jbrd),
            EncoderFailure::Jbrd
        );
        let x = EncoderFailure::Unspecified;
        println!("{x}, {x:?}");
    }
}
