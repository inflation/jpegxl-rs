use half::f16;
use image::ImageDecoder;

use crate::{
    decode::{Data, Metadata, PixelFormat},
    decoder_builder,
};
#[cfg(feature = "threads")]
use crate::{Endianness, ResizableRunner, ThreadsRunner};

#[test]
fn simple() {
    let mut decoder = decoder_builder().build().unwrap();

    let (
        Metadata {
            width,
            height,
            icc_profile,
            ..
        },
        data,
    ) = decoder
        .pixels()
        .with_icc_profile()
        .decode(super::SAMPLE_JXL)
        .unwrap();

    let Data::Uint16(data) = data else { panic!("Failed to decode") };

    assert_eq!(data.len(), (width * height * 4) as usize);
    // Check if icc profile is valid
    qcms::Profile::new_from_slice(&icc_profile.unwrap(), false)
        .expect("Failed to retrieve icc profile");
}

#[test]
fn pixel_types() {
    let mut decoder = decoder_builder().build().unwrap();

    // Check different pixel types
    let _ = decoder
        .pixels()
        .decode_to::<f32>(super::SAMPLE_JXL)
        .unwrap();
    let _ = decoder.pixels().decode_to::<u8>(super::SAMPLE_JXL).unwrap();
    let _ = decoder
        .pixels()
        .decode_to::<u16>(super::SAMPLE_JXL)
        .unwrap();
    let _ = decoder
        .pixels()
        .decode_to::<f16>(super::SAMPLE_JXL)
        .unwrap();
}

#[test]
fn jpeg() {
    let mut decoder = decoder_builder().init_jpeg_buffer(512).build().unwrap();

    let (_, data) = decoder.jpeg().reconstruct(super::SAMPLE_JXL_JPEG).unwrap();

    let jpeg = image::codecs::jpeg::JpegDecoder::new(data.as_slice())
        .expect("Failed to read the metadata of the reconstructed jpeg file");
    let mut v = vec![0; jpeg.total_bytes().try_into().unwrap()];
    jpeg.read_image(&mut v)
        .expect("Failed to read the reconstructed jpeg");

    let (_, jpeg, pixels) = decoder
        .jpeg()
        .reconstruct_with_pixels(super::SAMPLE_JXL)
        .unwrap();
    assert!(jpeg.is_none());
    assert!(matches!(pixels, Data::Uint16(_)));
}

#[test]
#[cfg(feature = "threads")]
fn builder() {
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
        .build()
        .unwrap();

    let (Metadata { width, height, .. }, data) = decoder
        .pixels()
        .decode_to::<f32>(super::SAMPLE_JXL)
        .unwrap();
    assert_eq!(data.len(), (width * height * 3) as usize);

    // Set options after creating decoder
    decoder.pixel_format = Some(PixelFormat {
        num_channels: 4,
        endianness: Endianness::Little,
        ..PixelFormat::default()
    });
    decoder.skip_reorientation = Some(true);
    decoder.parallel_runner = Some(&threads_runner);

    decoder.pixels().decode(super::SAMPLE_JXL).unwrap();
    let (Metadata { width, height, .. }, data) = decoder
        .pixels()
        .decode_to::<f32>(super::SAMPLE_JXL)
        .unwrap();
    assert_eq!(data.len(), (width * height * 4) as usize);
}
