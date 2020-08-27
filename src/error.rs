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

/// Errors derived from JpegxlDecoderStatus
#[derive(Debug)]
pub enum JpegxlError {
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

impl std::fmt::Display for JpegxlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for JpegxlError {}

/// Error mapping from underlying C const to JpegxlError enum
pub fn get_error(status: JpegxlDecoderStatus) -> Result<(), JpegxlError> {
    match status {
        JpegxlDecoderStatus_JPEGXL_DEC_ERROR => Err(JpegxlError::GenericError),
        JpegxlDecoderStatus_JPEGXL_DEC_NEED_MORE_INPUT => Err(JpegxlError::NeedMoreInput),
        JpegxlDecoderStatus_JPEGXL_DEC_NEED_MORE_OUTPUT => Err(JpegxlError::NeedMoreOutput),
        _ => Ok(()),
    }
}
