use half::f16;
use image::ImageDecoder;
use jpegxl_rs::{decode::DecoderResult, decoder_builder, DecodeError, Endianness, ThreadsRunner};

#[test]
fn simple() {
    let sample = std::fs::read("../samples/sample.jxl").unwrap();
    let decoder = decoder_builder().build().unwrap();

    let DecoderResult {
        width,
        height,
        icc_profile,
        data,
        ..
    } = decoder.decode(&sample).unwrap();

    let data = data.as_u16().unwrap();

    assert_eq!(data.len(), (width * height * 4) as usize);
    // Check if icc profile is valid
    qcms::Profile::new_from_slice(&icc_profile).expect("Failed to retrieve icc profile");
}

#[test]
fn pixel_types() {
    let sample = std::fs::read("../samples/sample.jxl").unwrap();
    let decoder = decoder_builder().build().unwrap();

    // Check different pixel types
    let DecoderResult { data, .. } = decoder.decode_to::<u8>(&sample).unwrap();
    let _data = data.as_u8().unwrap();

    let DecoderResult { data, .. } = decoder.decode_to::<f16>(&sample).unwrap();
    let _data = data.as_f16().unwrap();

    let DecoderResult { data, .. } = decoder.decode_to::<f32>(&sample).unwrap();
    let _data = data.as_f32().unwrap();

    // Check different pixel format but failed
    let DecoderResult { data, .. } = decoder.decode(&sample).unwrap();
    assert!(data.as_u8().is_none());

    let DecoderResult { data, .. } = decoder.decode_to::<u8>(&sample).unwrap();
    assert!(data.as_u16().is_none());

    let DecoderResult { data, .. } = decoder.decode_to::<u8>(&sample).unwrap();
    assert!(data.as_f16().is_none());

    let DecoderResult { data, .. } = decoder.decode_to::<u8>(&sample).unwrap();
    assert!(data.as_f32().is_none());
}

#[test]
fn container() {
    let sample = std::fs::read("../samples/sample_jpg.jxl").unwrap();
    let decoder = decoder_builder().init_jpeg_buffer(512).build().unwrap();

    let (_, data) = decoder.decode_jpeg(&sample).unwrap();

    let jpeg = image::codecs::jpeg::JpegDecoder::new(data.as_slice())
        .expect("Failed to read the metadata of the reconstructed jpeg file");
    let mut v = vec![0; jpeg.total_bytes().try_into().unwrap()];
    jpeg.read_image(&mut v)
        .expect("Failed to read the reconstructed jpeg");

    let sample = std::fs::read("../samples/sample.jxl").unwrap();
    assert!(matches!(
        decoder.decode_jpeg(&sample),
        Err(DecodeError::CannotReconstruct)
    ));
}

#[test]
fn builder() {
    let sample = std::fs::read("../samples/sample.jxl").unwrap();
    let parallel_runner = ThreadsRunner::default();
    let mut decoder = decoder_builder()
        .num_channels(3)
        .endianness(Endianness::Big)
        .align(2)
        .keep_orientation(true)
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();

    let DecoderResult {
        width,
        height,
        data,
        ..
    } = decoder.decode_to::<f32>(&sample).unwrap();

    let data = data.as_f32().unwrap();
    assert_eq!(data.len(), (width * height * 3) as usize);

    decoder.align = 0;
    decoder.endianness = Endianness::Native;
    decoder.num_channels = 4;
    decoder.keep_orientation = true;

    decoder.decode(&sample).unwrap();
}
