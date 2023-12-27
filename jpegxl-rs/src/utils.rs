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

//! Utils functions when a decoder or encoder is not needed

use jpegxl_sys::decode::{JxlSignature, JxlSignatureCheck};

/// Check if the signature of the input is valid.
/// Return `None` if it needs more data.
#[must_use]
pub fn check_valid_signature(buf: &[u8]) -> Option<bool> {
    use JxlSignature::{Codestream, Container, Invalid, NotEnoughBytes};

    match unsafe { JxlSignatureCheck(buf.as_ptr(), buf.len()) } {
        NotEnoughBytes => None,
        Invalid => Some(false),
        Codestream | Container => Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::SAMPLE_JXL;

    #[test]
    fn test_signature() {
        assert!(check_valid_signature(&[]).is_none());
        assert_eq!(check_valid_signature(&[0; 64]), Some(false));
        assert_eq!(check_valid_signature(SAMPLE_JXL), Some(true));
    }
}
