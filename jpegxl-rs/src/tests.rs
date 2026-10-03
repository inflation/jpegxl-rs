mod decode;
mod encode;

pub const SAMPLE_PNG: &[u8] = include_bytes!("../../samples/sample.png");
pub(crate) const SAMPLE_JPEG: &[u8] = include_bytes!("../../samples/sample.jpg");
const SAMPLE_JPEG_CMYK: &[u8] = include_bytes!("../../samples/sample_cmyk.jpg");
pub(crate) const SAMPLE_EXIF: &[u8] = include_bytes!("../../samples/sample.exif");
pub(crate) const SAMPLE_XMP: &[u8] = include_bytes!("../../samples/sample.xmp");
pub const SAMPLE_JXL: &[u8] = include_bytes!("../../samples/sample.jxl");
pub(crate) const SAMPLE_JXL_JPEG: &[u8] = include_bytes!("../../samples/sample_jpg.jxl");
pub const SAMPLE_JXL_GRAY: &[u8] = include_bytes!("../../samples/sample_grey.jxl");
const SAMPLE_JXL_2BIT: &[u8] = include_bytes!("../../samples/2bit.jxl");
pub(crate) const SAMPLE_ANIMATION: &[u8] = include_bytes!("../../samples/animation.jxl");
pub(crate) const SAMPLE_LAYERS: &[u8] = include_bytes!("../../samples/layers.jxl");
pub(crate) const SAMPLE_PROGRESSIVE: &[u8] = include_bytes!("../../samples/progressive.jxl");
pub(crate) const SAMPLE_EXTRA_CHANNEL: &[u8] = include_bytes!("../../samples/extra_channel.jxl");
pub(crate) const SAMPLE_BOXES: &[u8] = include_bytes!("../../samples/boxes.jxl");
