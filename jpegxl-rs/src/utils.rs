//! Utils functions when a decoder or encoder is not needed

use jpegxl_sys::{JxlSignature, JxlSignatureCheck};

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
        assert!(matches!(check_valid_signature(&[]), None));
        assert_eq!(check_valid_signature(&[0; 64]), Some(false));
        assert_eq!(check_valid_signature(SAMPLE_JXL), Some(true));
    }
}
