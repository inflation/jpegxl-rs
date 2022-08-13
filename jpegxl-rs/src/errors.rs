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

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::*;
use thiserror::Error;

/// Errors derived from [`JxlDecoderStatus`]
#[derive(Error, Debug)]
pub enum DecodeError {
    /// Unable to read more data
    #[error(transparent)]
    InputError(#[from] std::io::Error),
    /// Cannot create a decoder
    #[error("Cannot create a decoder")]
    CannotCreateDecoder,
    /// Unknown Error
    #[error("Generic Error")]
    GenericError,
    /// Invalid file format
    #[error("The file does not contain a valid codestream or container")]
    InvalidFileFormat,
    /// Cannot reconstruct JPEG codestream
    #[error("Cannot reconstruct JPEG codestream from the file")]
    CannotReconstruct,
    /// Unknown status
    #[error("Unknown status: `{0:?}`")]
    UnknownStatus(JxlDecoderStatus),
}

/// Errors derived from [`JxlEncoderStatus`]
#[derive(Error, Debug)]
pub enum EncodeError {
    /// Cannot create an encoder
    #[error("Cannot create an encoder")]
    CannotCreateEncoder,
    /// Generic Error
    #[error("Generic Error")]
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
    /// The encoder API is used in an incorrect way. In this case, a debug build of libjxl should output a specific error message
    #[error("The encoder API is used in an incorrect way")]
    ApiUsage,
    /// Unknown status
    #[error("Unknown status: `{0:?}`")]
    UnknownStatus(u32),
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
    use super::*;

    #[test]
    fn decode_invalid_data() {
        let sample = Vec::new();

        let decoder = crate::decoder_builder()
            .build()
            .expect("Failed to create decoder");
        assert!(matches!(
            decoder.decode_to::<u8>(&sample),
            Err(DecodeError::InvalidFileFormat)
        ));
    }

    #[test]
    fn encode_invalid_data() {
        let mut encoder = crate::encoder_builder()
            .has_alpha(true)
            .build()
            .expect("Failed to create encoder");

        assert!(matches!(
            encoder.encode::<u8, u8>(&[], 0, 0),
            Err(EncodeError::ApiUsage)
        ));

        assert!(matches!(
            encoder.encode::<f32, f32>(&[1.0, 1.0, 1.0, 0.5], 1, 1),
            Err(EncodeError::ApiUsage)
        ));
    }
}
