use half::f16;
use image::ImageDecoder;
use testresult::TestResult;

use crate::{
    decode::{Data, Metadata, PixelFormat, Pixels},
    decoder_builder,
};
#[cfg(feature = "threads")]
use crate::{Endianness, ResizableRunner, ThreadsRunner};

#[test]
fn simple() -> TestResult {
    let decoder = decoder_builder().icc_profile(true).build()?;

    let (
        Metadata {
            width,
            height,
            icc_profile,
            ..
        },
        data,
    ) = decoder.decode(super::SAMPLE_JXL)?;

    let Pixels::Uint16(data) = data else { return Err("Failed to decode".into()) };

    assert_eq!(data.len(), (width * height * 4) as usize);
    // Check if icc profile is valid
    qcms::Profile::new_from_slice(&icc_profile.unwrap(), false)
        .ok_or("Failed to retrieve icc profile")?;

    Ok(())
}

#[test]
fn pixel_types() -> TestResult {
    let decoder = decoder_builder().build()?;

    // Check different pixel types
    decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    decoder.decode_with::<u8>(super::SAMPLE_JXL)?;
    decoder.decode_with::<u16>(super::SAMPLE_JXL)?;
    decoder.decode_with::<f16>(super::SAMPLE_JXL)?;

    Ok(())
}

#[test]
fn jpeg() -> TestResult {
    let decoder = decoder_builder().init_jpeg_buffer(512).build()?;

    let (_, data) = decoder.reconstruct(super::SAMPLE_JXL_JPEG)?;
    let Data::Jpeg(data) = data else { return Err("Failed to reconstruct".into()) };

    let jpeg = image::codecs::jpeg::JpegDecoder::new(data.as_slice())?;
    let mut v = vec![0; jpeg.total_bytes().try_into().unwrap()];
    jpeg.read_image(&mut v)?;

    let (_, data) = decoder.reconstruct(super::SAMPLE_JXL)?;
    assert!(matches!(data, Data::Pixels(Pixels::Uint16(_))));

    Ok(())
}

#[test]
#[cfg(feature = "threads")]
fn builder() -> TestResult {
    let threads_runner = ThreadsRunner::default();
    let resizable_runner = ResizableRunner::default();
    let mut decoder = decoder_builder()
        .pixel_format(PixelFormat {
            num_channels: 3,
            endianness: Endianness::Big,
            ..PixelFormat::default()
        })
        .skip_reorientation(true)
        .parallel_runner(&resizable_runner)
        .build()?;

    let (Metadata { width, height, .. }, data) = decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    assert_eq!(data.len(), (width * height * 3) as usize);

    // Set options after creating decoder
    decoder.pixel_format = Some(PixelFormat {
        num_channels: 4,
        endianness: Endianness::Little,
        ..PixelFormat::default()
    });
    decoder.skip_reorientation = Some(true);
    decoder.parallel_runner = Some(&threads_runner);

    decoder.decode(super::SAMPLE_JXL)?;
    let (Metadata { width, height, .. }, data) = decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    assert_eq!(data.len(), (width * height * 4) as usize);

    Ok(())
}
