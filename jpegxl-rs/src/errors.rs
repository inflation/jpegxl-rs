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

use thiserror::Error;

use jpegxl_sys::{decode::JxlDecoderStatus, encoder::encode::JxlEncoderError};

/// Errors derived from [`JxlDecoderStatus`]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DecodeError {
    /// Cannot create a decoder
    #[error("Cannot create a decoder")]
    CannotCreateDecoder,
    /// Unknown Error
    #[error(
        "Generic Error. Please build `libjxl` from source (using `vendored` feature) 
        in debug mode to get more information. Check `stderr` for any internal error messages."
    )]
    GenericError,
    /// Invalid input
    #[error("The input does not contain a valid codestream or container")]
    InvalidInput,
    /// Unsupported Pixel bit width
    #[error("Unsupported Pixel bit width: {0}")]
    UnsupportedBitWidth(u32),
    /// Internal error, usually invalid usages of the `libjxl` library
    #[error("Internal error, please file an issus: {0}")]
    InternalError(&'static str),
    /// Unknown status
    #[error("Unknown status: `{0:?}`")]
    UnknownStatus(JxlDecoderStatus),
}

/// Errors derived from [`JxlEncoderStatus`][jpegxl_sys::encoder::encode::JxlEncoderStatus]
/// and [`JxlEncoderError`]
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum EncodeError {
    /// Cannot create an encoder
    #[error("Cannot create an encoder")]
    CannotCreateEncoder,
    /// Generic Error
    #[error(
        "Generic Error. Please build `libjxl` from source (using `vendored` feature) 
        in debug mode to get more information. Check `stderr` for any internal error messages."
    )]
    GenericError,
    /// Not Supported
    #[error("Encoder does not support it (yet)")]
    NotSupported,
    /// Need more output
    #[error("Need more output")]
    NeedMoreOutput,
    /// Out of memory
    #[error("Out of memory")]
    OutOfMemory,
    /// JPEG bitstream reconstruction data could not be represented (e.g. too much tail data)
    #[error("JPEG bitstream reconstruction data could not be represented")]
    Jbrd,
    /// Input is invalid (e.g. corrupt JPEG file or ICC profile)
    #[error("Input is invalid")]
    BadInput,
    /// The encoder API is used in an incorrect way. In this case,
    /// a debug build of libjxl should output a specific error message
    #[error("The encoder API is used in an incorrect way")]
    ApiUsage,
    /// Unknown status
    #[error("Unknown status: `{0:?}`")]
    UnknownStatus(JxlEncoderError),
}

/// Error mapping from underlying C const to [`DecodeError`] enum
pub(crate) fn check_dec_status(status: JxlDecoderStatus) -> Result<(), DecodeError> {
    match status {
        JxlDecoderStatus::Success => Ok(()),
        JxlDecoderStatus::Error => Err(DecodeError::GenericError),
        _ => Err(DecodeError::UnknownStatus(status)),
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use crate::encode::JxlEncoder;

    use super::*;

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn decode_invalid_data() -> TestResult {
        let decoder = crate::decoder_builder().build()?;
        assert!(matches!(
            decoder.decode_with::<u8>(&[]),
            Err(DecodeError::InvalidInput)
        ));
        assert!(matches!(
            decoder.decode_with::<u8>(&[0; 64]),
            Err(DecodeError::InvalidInput)
        ));
        assert!(matches!(
            decoder.decode(&crate::tests::SAMPLE_JXL[..100]),
            Err(DecodeError::GenericError)
        ));

        assert!(matches!(
            check_dec_status(JxlDecoderStatus::Error),
            Err(DecodeError::GenericError)
        ));

        println!(
            "{x}, {x:?}",
            x = check_dec_status(JxlDecoderStatus::BasicInfo).unwrap_err()
        );

        Ok(())
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn encode_invalid_data() -> TestResult {
        let mut encoder = JxlEncoder::builder().has_alpha(true).build()?;

        println!("{}", encoder.encode::<u8, u8>(&[], 0, 0).err().unwrap());

        assert!(matches!(
            encoder.encode::<u8, u8>(&[], 0, 0),
            Err(EncodeError::ApiUsage)
        ));
        assert!(matches!(
            encoder.encode::<f32, f32>(&[1.0, 1.0, 1.0, 0.5], 1, 1),
            Err(EncodeError::ApiUsage)
        ));

        println!(
            "{x}, {x:?}",
            x = EncodeError::UnknownStatus(JxlEncoderError::OK)
        );

        Ok(())
    }
}
