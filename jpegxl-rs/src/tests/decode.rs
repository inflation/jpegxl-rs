use half::f16;
use image::ImageDecoder;

use crate::{decode::DecoderResult, decoder_builder};
#[cfg(feature = "threads")]
use crate::{Endianness, ResizableRunner, ThreadsRunner};

#[test]
fn simple() {
    let decoder = decoder_builder().build().unwrap();

    let DecoderResult {
        width,
        height,
        icc_profile,
        data,
        ..
    } = decoder.decode(super::SAMPLE_JXL).unwrap();

    let data = data.as_u16().unwrap();

    assert_eq!(data.len(), (width * height * 4) as usize);
    // Check if icc profile is valid
    qcms::Profile::new_from_slice(&icc_profile, false).expect("Failed to retrieve icc profile");
}

#[test]
fn pixel_types() {
    let decoder = decoder_builder().build().unwrap();

    // Check different pixel types
    let DecoderResult { data, .. } = decoder.decode_to::<u8>(super::SAMPLE_JXL).unwrap();
    let _data = data.as_u8().unwrap();

    let DecoderResult { data, .. } = decoder.decode_to::<f16>(super::SAMPLE_JXL).unwrap();
    let _data = data.as_f16().unwrap();

    let DecoderResult { data, .. } = decoder.decode_to::<f32>(super::SAMPLE_JXL).unwrap();
    let _data = data.as_f32().unwrap();

    // Check different pixel format but failed
    let DecoderResult { data, .. } = decoder.decode(super::SAMPLE_JXL).unwrap();
    assert!(data.as_u8().is_none());

    let DecoderResult { data, .. } = decoder.decode_to::<u8>(super::SAMPLE_JXL).unwrap();
    assert!(data.as_u16().is_none());

    let DecoderResult { data, .. } = decoder.decode_to::<u8>(super::SAMPLE_JXL).unwrap();
    assert!(data.as_f16().is_none());

    let DecoderResult { data, .. } = decoder.decode_to::<u8>(super::SAMPLE_JXL).unwrap();
    assert!(data.as_f32().is_none());
}

#[test]
fn jpeg() {
    let decoder = decoder_builder().init_jpeg_buffer(512).build().unwrap();

    let (_, data) = decoder.decode_jpeg(super::SAMPLE_JXL_JPEG).unwrap();

    let jpeg = image::codecs::jpeg::JpegDecoder::new(data.as_slice())
        .expect("Failed to read the metadata of the reconstructed jpeg file");
    let mut v = vec![0; jpeg.total_bytes().try_into().unwrap()];
    jpeg.read_image(&mut v)
        .expect("Failed to read the reconstructed jpeg");

    let (res, _) = decoder.decode_jpeg(super::SAMPLE_JXL).unwrap();
    let _ = res.data.as_u16().unwrap();
}

#[test]
#[cfg(feature = "threads")]
fn builder() {
    let threads_runner = ThreadsRunner::default();
    let resizable_runner = ResizableRunner::default();
    let mut decoder = decoder_builder()
        .num_channels(3)
        .endianness(Endianness::Big)
        .align(2)
        .keep_orientation(true)
        .parallel_runner(&resizable_runner)
        .build()
        .unwrap();

    let DecoderResult {
        width,
        height,
        data,
        ..
    } = decoder.decode_to::<f32>(super::SAMPLE_JXL).unwrap();

    let data = data.as_f32().unwrap();
    assert_eq!(data.len(), (width * height * 3) as usize);

    decoder.align = 0;
    decoder.endianness = Endianness::Native;
    decoder.num_channels = 4;
    decoder.keep_orientation = true;
    decoder.parallel_runner = Some(&threads_runner);

    decoder.decode(super::SAMPLE_JXL).unwrap();
}
