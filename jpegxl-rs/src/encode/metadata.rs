use jpegxl_sys::types::JxlBoxType;

/// Metadata box
pub enum Metadata<'d> {
    /// EXIF
    /// The contents of this box must be prepended by a 4-byte tiff header offset,
    /// which may be 4 zero bytes in case the tiff header follows immediately.
    Exif(&'d [u8]),
    /// XMP/IPTC metadata
    Xmp(&'d [u8]),
    /// JUMBF superbox
    Jumb(&'d [u8]),
    /// Custom Metadata.
    /// Type should not start with `jxl`, `JXL`, or conflict with other box type,
    /// and should be registered with MP4RA (mp4ra.org).
    Custom([u8; 4], &'d [u8]),
}

impl Metadata<'_> {
    #[must_use]
    pub(crate) fn box_type(t: [u8; 4]) -> JxlBoxType {
        JxlBoxType(unsafe { std::mem::transmute(t) })
    }
}
