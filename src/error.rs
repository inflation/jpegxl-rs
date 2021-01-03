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

#![allow(non_upper_case_globals)]

use jpegxl_sys::*;

/// Errors derived from JxlDecoderStatus
#[derive(Debug)]
pub enum JxlError {
    /// Cannot create a decoder
    CannotCreateDecoder,
    /// Unknown Error
    // TODO: underlying library is working on a way to retrieve error message
    GenericError,
    /// Need more input bytes
    NeedMoreInput,
    /// Need more output buffer
    NeedMoreOutput,
}

impl std::fmt::Display for JxlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for JxlError {}

/// Error mapping from underlying C const to JxlError enum
pub fn get_error(status: JxlDecoderStatus) -> Result<(), JxlError> {
    match status {
        s if s == JxlDecoderStatus_JXL_DEC_ERROR => Err(JxlError::GenericError),
        s if s == JxlDecoderStatus_JXL_DEC_NEED_MORE_INPUT => Err(JxlError::NeedMoreInput),
        _ => Ok(()),
    }
}
