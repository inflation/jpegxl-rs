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
    /// Cannot create a decoder
    #[error("Cannot create a decoder")]
    CannotCreateDecoder,
    /// Unknown Error
    // TODO: underlying library is working on a way to retrieve error message
    #[error("Unknown decoder error")]
    GenericError,
    /// Need more input bytes
    #[error("Doesn't need more input")]
    NeedMoreInput,
    /// Invalid file format
    #[error("The file does not contain a valid codestream or container")]
    InvalidFileFormat,
    /// Unknown status
    #[error("Unknown status: `{0:?}`")]
    UnknownStatus(JxlDecoderStatus),
}

/// Errors derived from [`JxlEncoderStatus`]
#[derive(Error, Debug)]
pub enum EncodeError {
    /// Cannot create a decoder
    #[error("Cannot create an encoder")]
    CannotCreateEncoder,
    /// Unknown Error
    // TODO: underlying library is working on a way to retrieve error message
    #[error("Unknown encoder error")]
    GenericError,
    /// Not Supported
    #[error("Encoder does not support (yet)")]
    NotSupported,
    /// Unknown status
    #[error("Unknown status: `{0:?}`")]
    UnknownStatus(JxlEncoderStatus),
}

/// Error mapping from underlying C const to `JxlDecoderStatus` enum
pub(crate) fn check_dec_status(status: JxlDecoderStatus) -> Result<(), DecodeError> {
    match status {
        JxlDecoderStatus::Success => Ok(()),
        JxlDecoderStatus::Error => Err(DecodeError::GenericError),
        _ => Err(DecodeError::UnknownStatus(status)),
    }
}

/// Error mapping from underlying C const to `JxlEncoderStatus` enum
pub(crate) fn check_enc_status(status: JxlEncoderStatus) -> Result<(), EncodeError> {
    match status {
        JxlEncoderStatus::Success => Ok(()),
        JxlEncoderStatus::Error => Err(EncodeError::GenericError),
        JxlEncoderStatus::NotSupported => Err(EncodeError::NotSupported),
        JxlEncoderStatus::NeedMoreOutput => Err(EncodeError::UnknownStatus(status)),
    }
}
