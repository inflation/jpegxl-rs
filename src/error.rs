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

// #![allow(non_upper_case_globals)]
// #![allow(non_camel_case_types)]
// #![allow(non_snake_case)]

use jpegxl_sys::*;

/// Errors derived from JxlDecoderStatus
#[derive(Debug)]
pub enum DecodeError {
    /// Cannot create a decoder
    CannotCreateDecoder,
    /// Unknown Error
    // TODO: underlying library is working on a way to retrieve error message
    GenericError,
    /// Need more input bytes
    NeedMoreInput,
    /// Unknown status
    UnknownStatus(JxlDecoderStatus),
}

/// Errors derived from JxlEncoderStatus
#[derive(Debug)]
pub enum EncodeError {
    /// Unknown Error
    // TODO: underlying library is working on a way to retrieve error message
    GenericError,
    /// Need more input bytes
    NeedMoreOutput,
    /// Unknown status
    UnknownStatus(JxlEncoderStatus),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for DecodeError {}
impl std::error::Error for EncodeError {}

/// Error mapping from underlying C const to JxlDecoderStatus enum
pub(crate) fn check_dec_status(status: JxlDecoderStatus) -> Result<(), DecodeError> {
    match status {
        JxlDecoderStatus_JXL_DEC_SUCCESS => Ok(()),
        JxlDecoderStatus_JXL_DEC_ERROR => Err(DecodeError::GenericError),
        JxlDecoderStatus_JXL_DEC_NEED_MORE_INPUT => Err(DecodeError::NeedMoreInput),
        _ => Err(DecodeError::UnknownStatus(status)),
    }
}

/// Error mapping from underlying C const to JxlEncoderStatus enum
pub(crate) fn check_enc_status(status: JxlEncoderStatus) -> Result<(), EncodeError> {
    match status {
        JxlEncoderStatus_JXL_ENC_SUCCESS => Ok(()),
        JxlEncoderStatus_JXL_ENC_ERROR => Err(EncodeError::GenericError),
        JxlEncoderStatus_JXL_ENC_NEED_MORE_OUTPUT => Err(EncodeError::NeedMoreOutput),
        _ => Err(EncodeError::UnknownStatus(status)),
    }
}
