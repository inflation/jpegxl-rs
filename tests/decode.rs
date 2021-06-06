use core::panic;

use anyhow::{Context, Result};
use image::ImageDecoder;
use jpegxl_rs::{
    decode::{Data, DecoderResult},
    decoder_builder, DecodeError, Endianness, ThreadsRunner,
};

#[test]
fn simple() -> Result<()> {
    let sample = std::fs::read("samples/sample.jxl")?;
    let parallel_runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&parallel_runner)
        .build()?;

    if let DecoderResult {
        info,
        data: Data::U8(data),
    } = decoder.decode_to::<u8>(&sample)?
    {
        assert_eq!(data.len(), (info.width * info.height * 4) as usize);
        // Check if icc profile is valid
        qcms::Profile::new_from_slice(&info.icc_profile)
            .context("Failed to retrieve ICC profile")?;

        Ok(())
    } else {
        panic!()
    }
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn container() -> Result<()> {
    let sample = std::fs::read("samples/sample_jpg.jxl")?;
    let parallel_runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&parallel_runner)
        .init_jpeg_buffer(512)
        .build()?;

    let (_, data) = decoder.decode_jpeg(&sample)?;

    let jpeg = image::codecs::jpeg::JpegDecoder::new(data.as_slice())?;
    let mut v = vec![0; jpeg.total_bytes() as usize];
    jpeg.read_image(&mut v)?;

    let sample = std::fs::read("samples/sample.jxl")?;
    assert!(matches!(
        decoder.decode_jpeg(&sample),
        Err(DecodeError::CannotReconstruct)
    ));

    Ok(())
}

#[test]
fn builder() -> Result<()> {
    let sample = std::fs::read("samples/sample.jxl")?;
    let parallel_runner = ThreadsRunner::default();
    let mut decoder = decoder_builder()
        .num_channels(3)
        .endianness(Endianness::Big)
        .align(2)
        .keep_orientation(true)
        .parallel_runner(&parallel_runner)
        .build()?;

    if let DecoderResult {
        info,
        data: Data::U8(data),
    } = decoder.decode_to::<u8>(&sample)?
    {
        assert_eq!(data.len(), (info.width * info.height * 3) as usize);
    } else {
        panic!()
    };

    decoder.align = 0;
    decoder.endianness = Endianness::Native;
    decoder.num_channels = 4;
    decoder.keep_orientation = true;

    decoder.decode_to::<u8>(&sample)?;

    Ok(())
}
