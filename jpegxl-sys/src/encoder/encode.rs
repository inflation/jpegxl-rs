/*
This file is part of jpegxl-sys.

jpegxl-sys is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-sys is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-sys.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Encoding API for JPEG XL.

use std::ffi::{c_char, c_void};

use super::stats::JxlEncoderStats;
use crate::{
    color::{cms_interface::JxlCmsInterface, color_encoding::JxlColorEncoding},
    common::memory_manager::JxlMemoryManager,
    common::types::{JxlBitDepth, JxlBool, JxlBoxType, JxlPixelFormat},
    metadata::codestream_header::{
        JxlBasicInfo, JxlBlendInfo, JxlExtraChannelInfo, JxlExtraChannelType, JxlFrameHeader,
    },
    threads::parallel_runner::JxlParallelRunner,
};

#[cfg(doc)]
use crate::{    
    common::types::{JxlBitDepthType, JxlDataType},
    encoder::stats::JxlEncoderStatsCreate,
};

/// Opaque structure that holds the JPEG XL encoder.
///
/// Allocated and initialized with [`JxlEncoderCreate`].
/// Cleaned up and deallocated with [`JxlEncoderDestroy`].
#[repr(C)]
pub struct JxlEncoder {
    _unused: [u8; 0],
}

/// Settings and metadata for a single image frame. This includes encoder options
/// for a frame such as compression quality and speed.
///
/// Allocated and initialized with [`JxlEncoderFrameSettingsCreate`].
/// Cleaned up and deallocated when the encoder is destroyed with
/// [`JxlEncoderDestroy`].
#[repr(C)]
pub struct JxlEncoderFrameSettings {
    _unused: [u8; 0],
}

/// Return value for multiple encoder functions.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlEncoderStatus {
    /// Function call finished successfully, or encoding is finished and there is
    /// nothing more to be done
    Success = 0,
    /// An error occurred, for example out of memory.
    Error = 1,
    /// The encoder needs more output buffer to continue encoding.
    NeedMoreOutput = 2,
}

/// Error conditions:
/// API usage errors have the 0x80 bit set to 1
/// Other errors have the 0x80 bit set to 0
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlEncoderError {
    /// No error
    OK = 0,

    /// Generic encoder error due to unspecified cause
    Generic = 1,

    /// Out of memory. TODO: not really used
    OutOfMemory = 2,

    /// JPEG bitstream reconstruction data could not be represented (e.g. too much tail data)
    Jbrd = 3,

    /// Input is invalid (e.g. corrupt JPEG file or ICC profile)
    BadInput = 4,

    /// The encoder doesn't (yet) support this. Either no version of libjxl
    /// supports this, and the API is used incorrectly, or the libjxl version
    /// should have been checked before trying to do this.
    NotSupported = 0x80,

    /// The encoder API is used in an incorrect way.
    /// In this case, a debug build of libjxl should output a specific error
    /// message. (if not, please open an issue about it)
    ApiUsage = 0x81,
}

/// Id of encoder options for a frame. This includes options such as setting
/// encoding effort/speed or overriding the use of certain coding tools, for this
/// frame. This does not include non-frame related encoder options such as for
/// boxes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlEncoderFrameSettingId {
    /// Sets encoder effort/speed level without affecting decoding speed. Valid
    /// values are, from faster to slower speed: 1:lightning 2:thunder 3:falcon
    /// 4:cheetah 5:hare 6:wombat 7:squirrel 8:kitten 9:tortoise 10:glacier.
    /// Default: squirrel (7).
    Effort = 0,

    /// Sets the decoding speed tier for the provided options. Minimum is 0
    /// (slowest to decode, best quality/density), and maximum is 4 (fastest to
    /// decode, at the cost of some quality/density). Default is 0.
    DecodingSpeed = 1,

    /// Sets resampling option. If enabled, the image is downsampled before
    /// compression, and upsampled to original size in the decoder. Integer option,
    /// use -1 for the default behavior (resampling only applied for low quality),
    /// 1 for no downsampling (1x1), 2 for 2x2 downsampling, 4 for 4x4
    /// downsampling, 8 for 8x8 downsampling.
    Resampling = 2,

    /// Similar to [`Self::Resampling`], but for extra channels.
    /// Integer option, use -1 for the default behavior (depends on encoder
    /// implementation), 1 for no downsampling (1x1), 2 for 2x2 downsampling, 4 for
    /// 4x4 downsampling, 8 for 8x8 downsampling.
    ExtraChannelResampling = 3,

    /// Indicates the frame added with [`JxlEncoderAddImageFrame`] is already
    /// downsampled by the downsampling factor set with [`Self::Resampling`]. The input frame
    /// must then be given in the downsampled resolution, not the full image resolution.
    /// The downsampled resolution is given by `ceil(xsize / resampling)`, `ceil(ysize / resampling)`
    /// with `xsize` and `ysize` the dimensions given in the basic info, and `resampling`
    /// the factor set with [`Self::Resampling`].
    /// Use 0 to disable, 1 to enable. Default value is 0.
    AlreadyDownsampled = 4,

    /// Adds noise to the image emulating photographic film noise, the higher the
    /// given number, the grainier the image will be. As an example, a value of 100
    /// gives low noise whereas a value of 3200 gives a lot of noise. The default
    /// value is 0.
    PhotonNoise = 5,

    /// Enables adaptive noise generation. This setting is not recommended for
    /// use, please use [`Self::PhotonNoise`] instead.
    /// Use -1 for the default (encoder chooses), 0 to disable, 1 to enable.
    Noise = 6,

    /// Enables or disables dots generation. Use -1 for the default (encoder
    /// chooses), 0 to disable, 1 to enable.
    Dots = 7,

    /// Enables or disables patches generation. Use -1 for the default (encoder
    /// chooses), 0 to disable, 1 to enable.
    Patches = 8,

    /// Edge preserving filter level, -1 to 3. Use -1 for the default (encoder
    /// chooses), 0 to 3 to set a strength.
    Epf = 9,

    /// Enables or disables the gaborish filter. Use -1 for the default (encoder
    /// chooses), 0 to disable, 1 to enable.
    Gaborish = 10,

    /// Enables modular encoding. Use -1 for default (encoder
    /// chooses), 0 to enforce `VarDCT` mode (e.g. for photographic images), 1 to
    /// enforce modular mode (e.g. for lossless images).
    Modular = 11,

    /// Enables or disables preserving color of invisible pixels. Use -1 for the
    /// default (1 if lossless, 0 if lossy), 0 to disable, 1 to enable.
    KeepInvisible = 12,

    /// Determines the order in which 256x256 regions are stored in the codestream
    /// for progressive rendering. Use -1 for the encoder
    /// default, 0 for scanline order, 1 for center-first order.
    GroupOrder = 13,

    /// Determines the horizontal position of center for the center-first group
    /// order. Use -1 to automatically use the middle of the image, 0..xsize to
    /// specifically set it.
    GroupOrderCenterX = 14,

    /// Determines the center for the center-first group order. Use -1 to
    /// automatically use the middle of the image, 0..ysize to specifically set it.
    GroupOrderCenterY = 15,

    /// Enables or disables progressive encoding for modular mode. Use -1 for the
    /// encoder default, 0 to disable, 1 to enable.
    Responsive = 16,

    /// Set the progressive mode for the AC coefficients of `VarDCT`, using spectral
    /// progression from the DCT coefficients. Use -1 for the encoder default, 0 to
    /// disable, 1 to enable.
    ProgressiveAc = 17,

    /// Set the progressive mode for the AC coefficients of `VarDCT`, using
    /// quantization of the least significant bits. Use -1 for the encoder default,
    /// 0 to disable, 1 to enable.
    QprogressiveAc = 18,

    /// Set the progressive mode using lower-resolution DC images for `VarDCT`. Use
    /// -1 for the encoder default, 0 to disable, 1 to have an extra 64x64 lower
    /// resolution pass, 2 to have a 512x512 and 64x64 lower resolution pass.
    ProgressiveDc = 19,

    /// Use Global channel palette if the amount of colors is smaller than this
    /// percentage of range. Use 0-100 to set an explicit percentage, -1 to use the
    /// encoder default. Used for modular encoding.
    ChannelColorsGlobalPercent = 20,

    /// Use Local (per-group) channel palette if the amount of colors is smaller
    /// than this percentage of range. Use 0-100 to set an explicit percentage, -1
    /// to use the encoder default. Used for modular encoding.
    ChannelColorsGroupPercent = 21,

    /// Use color palette if amount of colors is smaller than or equal to this
    /// amount, or -1 to use the encoder default. Used for modular encoding.
    PaletteColors = 22,

    /// Enables or disables delta palette. Use -1 for the default (encoder
    /// chooses), 0 to disable, 1 to enable. Used in modular mode.
    LossyPalette = 23,

    /// Color transform for internal encoding: -1 = default, 0=XYB, 1=none (RGB),
    /// 2=YCbCr. The XYB setting performs the forward XYB transform. None and
    /// YCbCr both perform no transform, but YCbCr is used to indicate that the
    /// encoded data losslessly represents YCbCr values.
    ColorTransform = 24,

    /// Reversible color transform for modular encoding: -1=default, 0-41=RCT
    /// index, e.g. index 0 = none, index 6 = `YCoCg`.
    /// If this option is set to a non-default value, the RCT will be globally
    /// applied to the whole frame.
    /// The default behavior is to try several RCTs locally per modular group,
    /// depending on the speed and distance setting.
    ModularColorSpace = 25,

    /// Group size for modular encoding: -1=default, 0=128, 1=256, 2=512, 3=1024.
    ModularGroupSize = 26,

    /// Predictor for modular encoding. -1 = default, 0=zero, 1=left, 2=top,
    /// 3=avg0, 4=select, 5=gradient, 6=weighted, 7=topright, 8=topleft,
    /// 9=leftleft, 10=avg1, 11=avg2, 12=avg3, 13=toptop predictive average 14=mix
    /// 5 and 6, 15=mix everything.
    ModularPredictor = 27,

    /// Fraction of pixels used to learn MA trees as a percentage. -1 = default,
    /// 0 = no MA and fast decode, 50 = default value, 100 = all, values above
    /// 100 are also permitted. Higher values use more encoder memory.
    ModularMaTreeLearningPercent = 28,

    /// Number of extra (previous-channel) MA tree properties to use. -1 =
    /// default, 0-11 = valid values. Recommended values are in the range 0 to 3,
    /// or 0 to amount of channels minus 1 (including all extra channels, and
    /// excluding color channels when using `VarDCT` mode). Higher value gives slower
    /// encoding and slower decoding.
    ModularNbPrevChannels = 29,

    /// Enable or disable CFL (chroma-from-luma) for lossless JPEG recompression.
    /// -1 = default, 0 = disable CFL, 1 = enable CFL.
    JpegReconCfl = 30,

    /// Prepare the frame for indexing in the frame index box.
    /// 0 = ignore this frame (same as not setting a value),
    /// 1 = index this frame within the Frame Index Box.
    /// If any frames are indexed, the first frame needs to
    /// be indexed, too. If the first frame is not indexed, and
    /// a later frame is attempted to be indexed, [`JxlEncoderStatus::Error`] will occur.
    /// If non-keyframes, i.e., frames with cropping, blending or patches are
    /// attempted to be indexed, [`JxlEncoderStatus::Error`] will occur.
    IndexBox = 31,

    /// Sets brotli encode effort for use in JPEG recompression and
    /// compressed metadata boxes (brob). Can be -1 (default) or 0 (fastest) to 11
    /// (slowest). Default is based on the general encode effort in case of JPEG
    /// recompression, and 4 for brob boxes.
    BrotliEffort = 32,

    /// Enables or disables brotli compression of metadata boxes derived from
    /// a JPEG frame when using [`JxlEncoderAddJPEGFrame`]. This has no effect on
    /// boxes added using [`JxlEncoderAddBox`]. -1 = default, 0 = disable
    /// compression, 1 = enable compression.
    JpegCompressBoxes = 33,

    /// Control what kind of buffering is used, when using chunked image frames.
    /// -1 = default (let the encoder decide)
    /// 0 = buffers everything, basically the same as non-streamed code path
    /// (mainly for testing)
    /// 1 = buffers everything for images that are smaller than 2048 x 2048, and
    ///     uses streaming input and output for larger images
    /// 2 = uses streaming input and output for all images that are larger than
    ///     one group, i.e. 256 x 256 pixels by default
    /// 3 = currently same as 2
    ///
    /// When using streaming input and output the encoder minimizes memory usage at
    /// the cost of compression density. Also note that images produced with
    /// streaming mode might not be progressively decodeable.
    Buffering = 34,

    /// Keep or discard Exif metadata boxes derived from a JPEG frame when using
    /// [`JxlEncoderAddJPEGFrame`]. This has no effect on boxes added using
    /// [`JxlEncoderAddBox`]. When [`JxlEncoderStoreJPEGMetadata`] is set to 1,
    /// this option cannot be set to 0. Even when Exif metadata is discarded, the
    /// orientation will still be applied. 0 = discard Exif metadata, 1 = keep Exif
    /// metadata (default).
    JpegKeepExif = 35,

    /// Keep or discard XMP metadata boxes derived from a JPEG frame when using
    /// [`JxlEncoderAddJPEGFrame`]. This has no effect on boxes added using
    /// [`JxlEncoderAddBox`]. When [`JxlEncoderStoreJPEGMetadata`] is set to 1,
    /// this option cannot be set to 0. 0 = discard XMP metadata, 1 = keep XMP
    /// metadata (default).
    JpegKeepXmp = 36,

    /// Keep or discard JUMBF metadata boxes derived from a JPEG frame when using
    /// [`JxlEncoderAddJPEGFrame`]. This has no effect on boxes added using
    /// [`JxlEncoderAddBox`]. 0 = discard JUMBF metadata, 1 = keep JUMBF metadata
    /// (default).
    JpegKeepJumbf = 37,

    /// If this mode is disabled, the encoder will not make any image quality
    /// decisions that are computed based on the full image, but stored only once
    /// (e.g. the X quant multiplier in the frame header). Used mainly for testing
    /// equivalence of streaming and non-streaming code.
    /// 0 = disabled, 1 = enabled (default)
    UseFullImageHeuristics = 38,

    /// Disable perceptual optimizations. 0 = optimizations enabled (default), 1 =
    /// optimizations disabled.
    DisablePerceptualHeuristics = 39,

    /// Enum value not to be used as an option. This value is added to force the
    /// C compiler to have the enum to take a known size.
    FillEnum = 65535,
}

/// The [`JxlEncoderOutputProcessor`] structure provides an interface for the
/// encoder's output processing. Users of the library, who want to do streaming
/// encoding, should implement the required callbacks for buffering, writing,
/// seeking (if supported), and setting a finalized position during the encoding
/// process.
///
/// At a high level, the processor can be in one of two states:
/// - With an active buffer: This indicates that a buffer has been acquired using
///   `get_buffer` and encoded data can be written to it.
/// - Without an active buffer: In this state, no data can be written. A new
///   buffer must be acquired after releasing any previously active buffer.
///
/// The library will not acquire more than one buffer at a given time.
///
/// The state of the processor includes `position` and `finalized position`,
/// which have the following meaning.
///
/// - position: Represents the current position, in bytes, within the output
///   stream where the encoded data will be written next. This position moves
///   forward with each `release_buffer` call as data is written, and can also be
///   adjusted through the optional seek callback, if provided. At this position
///   the next write will occur.
///
/// - finalized position: A position in the output stream that ensures all bytes
///   before this point are finalized and won't be changed by later writes.
///
/// All fields but `seek` are required, `seek` is optional and can be `None`.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlEncoderOutputProcessor {
    /// An opaque pointer that the client can use to store custom data.
    /// This data will be passed to the associated callback functions.
    opaque: *mut c_void,
    /// Acquires a buffer at the current position into which the library will write
    /// the output data.
    ///
    /// If the `size` argument points to 0 and the returned value is NULL, this
    /// will be interpreted as asking the output writing to stop. In such a case,
    /// the library will return an error. The client is expected to set the size of
    /// the returned buffer based on the suggested `size` when this function is
    /// called.
    ///
    /// # Parameters
    /// - `opaque`: User supplied parameters to the callback.
    /// - `size`: Points to a suggested buffer size when called; must be set to
    ///   the size of the returned buffer once the function returns.
    ///
    /// # Returns
    /// A pointer to the acquired buffer or NULL to indicate a stop
    /// condition.
    get_buffer: extern "C-unwind" fn(opaque: *mut c_void, size: *mut usize) -> *mut c_void,
    /// Notifies the user of library that the current buffer's data has been
    /// written and can be released. This function should advance the current
    /// osition of the buffer by `written_bytes` number of bytes.
    ///
    /// # Parameters
    /// - `opaque`: user supplied parameters to the callback
    /// - `written_bytes`: the number of bytes written to the buffer.
    release_buffer: extern "C-unwind" fn(opaque: *mut c_void, written_bytes: usize),
    /// Seeks to a specific position in the output. This function is optional and
    /// can be set to `None` if the output doesn't support seeking. Can only be done
    /// when there is no buffer. Cannot be used to seek before the finalized
    /// position.
    ///
    /// # Parameters
    /// - `opaque`: User supplied parameters to the callback.
    /// - `position`: The position to seek to, in bytes.
    seek: Option<extern "C-unwind" fn(opaque: *mut c_void, position: u64)>,
    /// Sets a finalized position on the output data, at a specific position.
    /// Seeking will never request a position before the finalized position.
    ///
    /// Will only be called if there is no active buffer.
    ///
    /// # Parameters
    /// - `opaque`: User supplied parameters to the callback.
    /// - `finalized_position`: The position, in bytes, where the finalized
    ///   position should be set.
    set_finalized_position: extern "C-unwind" fn(opaque: *mut c_void, finalized_position: u64),
}

/// This struct provides callback functions to pass pixel data in a streaming
/// manner instead of requiring the entire frame data in memory at once.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlChunkedFrameInputSource {
    /// A pointer to any user-defined data or state. This can be used to pass
    /// information to the callback functions.
    opaque: *mut c_void,

    /// Get the pixel format that color channel data will be provided in.
    /// When called, `pixel_format` points to a suggested pixel format; if
    /// color channel data can be given in this pixel format, processing might
    /// be more efficient.
    ///
    /// This function will be called exactly once, before any call to
    /// [`Self::get_color_channels_data`].
    ///
    /// # Parameters
    /// - `opaque`: User supplied parameters to the callback.
    /// - `pixel_format`: Format for pixels.
    get_color_channels_pixel_format:
        extern "C-unwind" fn(opaque: *mut c_void, pixel_format: *mut JxlPixelFormat),

    /// Callback to retrieve a rectangle of color channel data at a specific
    /// location. It is guaranteed that `xpos` and `ypos` are multiples of 8. `xsize`,
    /// `ysize` will be multiples of 8, unless the resulting rectangle would be out
    /// of image bounds. Moreover, `xsize` and `ysize` will be at most 2048. The
    /// returned data will be assumed to be in the format returned by the
    /// (preceding) call to [`Self::get_color_channels_pixel_format`], except the `align`
    /// parameter of the pixel format will be ignored. Instead, the `i`-th row will
    /// be assumed to start at position `return_value + i * *row_offset`, with the
    /// value of `*row_offset` decided by the callee.
    ///
    /// Note that multiple calls to `get_color_channel_data_at` may happen before a
    /// call to [`Self::release_buffer`].
    ///
    /// # Parameters
    /// - `opaque`: User supplied parameters to the callback.
    /// - `xpos`: Horizontal position for the data.
    /// - `ypos`: Vertical position for the data.
    /// - `xsize`: Horizontal size of the requested rectangle of data.
    /// - `ysize`: Vertical size of the requested rectangle of data.
    /// - `row_offset`: Pointer to the byte offset between consecutive rows of
    ///   the retrieved pixel data.
    ///
    /// # Returns
    /// Pointer to the retrieved pixel data.
    get_color_channels_data_at: extern "C-unwind" fn(
        opaque: *mut c_void,
        xpos: usize,
        ypos: usize,
        xsize: usize,
        ysize: usize,
        row_offset: *mut usize,
    ) -> *const c_void,

    /// Get the pixel format that extra channel data will be provided in.
    /// When called, `pixel_format` points to a suggested pixel format; if
    /// extra channel data can be given in this pixel format, processing might
    /// be more efficient.
    ///
    /// This function will be called exactly once per index, before any call to
    /// [`Self::get_extra_channel_data_at`] with that given index.
    ///
    /// # Parameters
    /// - `opaque`: User supplied parameters to the callback.
    /// - `ec_index`: Zero-indexed index of the extra channel.
    /// - `pixel_format`: Format for extra channel data.
    get_extra_channel_pixel_format: extern "C-unwind" fn(
        opaque: *mut c_void,
        ec_index: usize,
        pixel_format: *mut JxlPixelFormat,
    ),

    /// Callback to retrieve a rectangle of extra channel `ec_index` data at a
    /// specific location. It is guaranteed that `xpos` and `ypos` are multiples of
    /// 8. `xsize`, `ysize` will be multiples of 8, unless the resulting rectangle
    /// would be out of image bounds. Moreover, `xsize` and `ysize` will be at most
    /// 2048. The returned data will be assumed to be in the format returned by the
    /// (preceding) call to [`Self::get_extra_channel_pixel_format`] with the
    /// corresponding extra channel index `ec_index`, except the `align` parameter
    /// of the pixel format will be ignored. Instead, the `i`-th row will be
    /// assumed to start at position `return_value + i * *row_offset`, with the
    /// value of `*row_offset` decided by the callee.
    ///
    /// Note that multiple calls to `get_extra_channel_data_at` may happen before a
    /// call to [`Self::release_buffer`].
    ///
    /// # Parameters
    /// - `opaque`: User supplied parameters to the callback.
    /// - `xpos`: Horizontal position for the data.
    /// - `ypos`: Vertical position for the data.
    /// - `xsize`: Horizontal size of the requested rectangle of data.
    /// - `ysize`: Vertical size of the requested rectangle of data.
    /// - `row_offset`: Pointer to the byte offset between consecutive rows of
    ///   the retrieved pixel data.
    ///
    /// # Returns
    /// Pointer to the retrieved pixel data.
    get_extra_channel_data_at: extern "C-unwind" fn(
        opaque: *mut c_void,
        ec_index: usize,
        xpos: usize,
        ypos: usize,
        xsize: usize,
        ysize: usize,
        row_offset: *mut usize,
    ) -> *const c_void,

    /// Releases the buffer `buf` (obtained through a call to
    /// [`Self::get_color_channels_data_at`] or [`Self::get_extra_channel_data_at`]). This function
    /// will be called exactly once per call to [`Self::get_color_channels_data_at`] or
    /// [`Self::get_extra_channel_data_at`].
    ///
    /// # Parameters
    /// - `opaque`: User supplied parameters to the callback.
    /// - `buf`: Pointer returned by [`Self::get_color_channels_data_at`] or
    ///   [`Self::get_extra_channel_data_at`].
    release_buffer: extern "C-unwind" fn(opaque: *mut c_void, buf: *const c_void),
}

/// Function type for [`JxlEncoderSetDebugImageCallback`].
///
/// The callback may be called simultaneously by different threads when using a
/// threaded parallel runner, on different debug images.
///
/// # Parameters
/// - `opaque`: Optional user data, as given to [`JxlEncoderSetDebugImageCallback`].
/// - `label`: Label of debug image, can be used in filenames.
/// - `xsize`: Width of debug image.
/// - `ysize`: Height of debug image.
/// - `color`: Color encoding of debug image.
/// - `pixels`: Pixel data of debug image as big-endian 16-bit unsigned samples.
///   The memory is not owned by the user, and is only valid during the time the
///   callback is running.
pub type JxlDebugImageCallback = extern "C-unwind" fn(
    opaque: *mut c_void,
    label: *const c_char,
    xsize: usize,
    ysize: usize,
    color: *const JxlColorEncoding,
    pixels: *const u16,
);

extern "C" {
    /// Encoder library version.
    ///
    /// # Returns
    /// The encoder library version as an integer:
    /// `MAJOR_VERSION * 1000000 + MINOR_VERSION * 1000 + PATCH_VERSION`.
    /// For example, version 1.2.3 would return 1002003.
    pub fn JxlEncoderVersion() -> u32;

    /// Creates an instance of `JxlEncoder` and initializes it.
    ///
    /// # Parameters
    /// - `memory_manager`: Custom allocator function. It may be `NULL`. The memory
    ///   manager will be copied internally. If `NULL`, the default allocator will be used.
    ///   See [`crate::common::memory_manager`] for details.
    ///
    /// # Returns
    /// - `NULL` if the instance cannot be allocated or initialized.
    /// - Pointer to initialized [`JxlEncoder`] otherwise.
    pub fn JxlEncoderCreate(memory_manager: *const JxlMemoryManager) -> *mut JxlEncoder;

    /// Re-initializes a [`JxlEncoder`] instance, so it can be re-used for encoding
    /// another image. All state and settings are reset as if the object was
    /// newly created with [`JxlEncoderCreate`], but the memory manager is kept.
    ///
    /// # Parameters
    /// - `enc`: Instance to be re-initialized.
    pub fn JxlEncoderReset(enc: *mut JxlEncoder);

    /// Deinitializes and frees a [`JxlEncoder`] instance.
    ///
    /// # Parameters
    /// - `enc`: Instance to be cleaned up and deallocated.
    pub fn JxlEncoderDestroy(enc: *mut JxlEncoder);

    /// Sets the color management system (CMS) that will be used for color conversion
    /// (if applicable) during encoding. May only be set before starting encoding. If
    /// left unset, the default CMS implementation will be used.
    ///
    /// # Parameters
    /// - `enc`: Encoder object.
    /// - `cms`: Structure representing a CMS implementation. See [`JxlCmsInterface`] for more details.
    pub fn JxlEncoderSetCms(enc: *mut JxlEncoder, cms: JxlCmsInterface);

    /// Set the parallel runner for multithreading. May only be set before starting
    /// encoding.
    ///
    /// # Parameters
    /// - `enc`: Encoder object.
    /// - `parallel_runner`: Function pointer to runner for multithreading. It may
    ///   be `NULL` to use the default, single-threaded, runner. A multithreaded
    ///   runner should be set to reach fast performance.
    /// - `parallel_runner_opaque`: Opaque pointer for `parallel_runner`.
    ///
    /// # Returns
    /// - `JxlEncoderStatus::Success` if the runner was set.
    /// - `JxlEncoderStatus::Error` otherwise (the previous runner remains set).
    pub fn JxlEncoderSetParallelRunner(
        enc: *mut JxlEncoder,
        parallel_runner: JxlParallelRunner,
        parallel_runner_opaque: *mut c_void,
    ) -> JxlEncoderStatus;

    /// Get the (last) error code in case [`JxlEncoderStatus::Error`] was returned.
    ///
    /// # Parameters
    /// - `enc`: Encoder object.
    ///
    /// # Returns
    /// The [`JxlEncoderError`] that caused the (last) [`JxlEncoderStatus::Error`] to be returned.
    pub fn JxlEncoderGetError(enc: *mut JxlEncoder) -> JxlEncoderError;

    /// Encodes a JPEG XL file using the available bytes. `*avail_out` indicates how
    /// many output bytes are available, and `*next_out` points to the input bytes.
    /// `*avail_out` will be decremented by the amount of bytes that have been
    /// processed by the encoder and `*next_out` will be incremented by the same
    /// amount, so `*next_out` will now point at the amount of `*avail_out` unprocessed
    /// bytes.
    ///
    /// The returned status indicates whether the encoder needs more output bytes.
    /// When the return value is not [`JxlEncoderStatus::Error`] or [`JxlEncoderStatus::Success`], the
    /// encoding requires more [`JxlEncoderProcessOutput`] calls to continue.
    ///
    /// The caller must guarantee that `*avail_out >= 32` when calling
    /// [`JxlEncoderProcessOutput`]; otherwise, [`JxlEncoderStatus::NeedMoreOutput`] will
    /// be returned. It is guaranteed that, if `*avail_out >= 32`, at least one byte of
    /// output will be written.
    ///
    /// This encodes the frames and/or boxes added so far. If the last frame or last
    /// box has been added, [`JxlEncoderCloseInput`], [`JxlEncoderCloseFrames`]
    /// and/or [`JxlEncoderCloseBoxes`] must be called before the next
    /// [`JxlEncoderProcessOutput`] call, or the codestream won't be encoded
    /// correctly.
    ///
    /// # Parameters
    /// - `enc`: Encoder object.
    /// - `next_out`: Pointer to next bytes to write to.
    /// - `avail_out`: Amount of bytes available starting from `*next_out`.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] when encoding finished and all events handled.
    /// - [`JxlEncoderStatus::Error`] when encoding failed, e.g. invalid input.
    /// - [`JxlEncoderStatus::NeedMoreOutput`] more output buffer is necessary.
    pub fn JxlEncoderProcessOutput(
        enc: *mut JxlEncoder,
        next_out: *mut *mut u8,
        avail_out: *mut usize,
    ) -> JxlEncoderStatus;

    /// Sets the frame information for this frame to the encoder. This includes
    /// animation information such as frame duration to store in the frame header.
    /// The frame header fields represent the frame as passed to the encoder, but not
    /// necessarily the exact values as they will be encoded file format: the encoder
    /// could change crop and blending options of a frame for more efficient encoding
    /// or introduce additional internal frames. Animation duration and time code
    /// information is not altered since those are immutable metadata of the frame.
    ///
    /// It is not required to use this function, however if `have_animation` is set
    /// to true in the basic info, then this function should be used to set the
    /// time duration of this individual frame. By default individual frames have a
    /// time duration of 0, making them form a composite still. See [`JxlFrameHeader`]
    /// for more information.
    ///
    /// This information is stored in the [`JxlEncoderFrameSettings`] and so is used
    /// for any frame encoded with these [`JxlEncoderFrameSettings`]. It is ok to
    /// change between [`JxlEncoderAddImageFrame`] calls, each added image frame
    /// will have the frame header that was set in the options at the time of calling
    /// [`JxlEncoderAddImageFrame`].
    ///
    /// The `is_last` and `name_length` fields of the [`JxlFrameHeader`] are ignored,
    /// use [`JxlEncoderCloseFrames`] to indicate last frame, and [`JxlEncoderSetFrameName`]
    /// to indicate the name and its length instead.
    /// Calling this function will clear any name that was previously set with [`JxlEncoderSetFrameName`].
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `frame_header`: frame header data to set. Object owned by the caller and
    ///   does not need to be kept in memory, its information is copied internally.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success
    /// - [`JxlEncoderStatus::Error`] on error
    pub fn JxlEncoderSetFrameHeader(
        frame_settings: *mut JxlEncoderFrameSettings,
        frame_header: *const JxlFrameHeader,
    ) -> JxlEncoderStatus;

    /// Sets blend info of an extra channel. The blend info of extra channels is set
    /// separately from that of the color channels, the color channels are set with
    /// [`JxlEncoderSetFrameHeader`].
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `index`: index of the extra channel to use.
    /// - `blend_info`: blend info to set for the extra channel.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success.
    /// - [`JxlEncoderStatus::Error`] on error.
    pub fn JxlEncoderSetExtraChannelBlendInfo(
        frame_settings: *mut JxlEncoderFrameSettings,
        index: usize,
        blend_info: *const JxlBlendInfo,
    ) -> JxlEncoderStatus;

    /// Sets the name of the animation frame. This function is optional, frames are
    /// not required to have a name. This setting is a part of the frame header, and
    /// the same principles as for [`JxlEncoderSetFrameHeader`] apply. The
    /// `name_length` field of [`JxlFrameHeader`] is ignored by the encoder, this
    /// function determines the name length instead as the length in bytes of the C
    /// string.
    ///
    /// The maximum possible name length is 1071 bytes (excluding terminating null
    /// character).
    ///
    /// Calling [`JxlEncoderSetFrameHeader`] clears any name that was
    /// previously set.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `frame_name`: name of the next frame to be encoded, as a UTF-8 encoded C
    ///   string (zero terminated). Owned by the caller, and copied internally.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success
    /// - [`JxlEncoderStatus::Error`] on error
    pub fn JxlEncoderSetFrameName(
        frame_settings: *mut JxlEncoderFrameSettings,
        frame_name: *const u8,
    ) -> JxlEncoderStatus;

    /// Sets the bit depth of the input buffer.
    ///
    /// For float pixel formats, only the default [`JxlBitDepthType::FromPixelFormat`]
    /// setting is allowed, while for unsigned pixel formats,
    /// [`JxlBitDepthType::FromCodestream`] setting is also allowed. See the comment
    /// on [`JxlEncoderAddImageFrame`] for the effects of the bit depth setting.
    ///
    /// # Parameters
    /// - `frame_settings`: Set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `bit_depth`: The bit depth setting of the pixel input.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success.
    /// - [`JxlEncoderStatus::Error`] on error.
    pub fn JxlEncoderSetFrameBitDepth(
        frame_settings: *mut JxlEncoderFrameSettings,
        bit_depth: *const JxlBitDepth,
    ) -> JxlEncoderStatus;

    /// Sets the buffer to read JPEG encoded bytes from for the next frame to encode.
    ///
    /// If [`JxlEncoderSetBasicInfo`] has not yet been called, calling
    /// [`JxlEncoderAddJPEGFrame`] will implicitly call it with the parameters of
    /// the added JPEG frame.
    ///
    /// If [`JxlEncoderSetColorEncoding`] or [`JxlEncoderSetICCProfile`] has not
    /// yet been called, calling [`JxlEncoderAddJPEGFrame`] will implicitly call it
    /// with the parameters of the added JPEG frame.
    ///
    /// If the encoder is set to store JPEG reconstruction metadata using [`JxlEncoderStoreJPEGMetadata`]
    /// and a single JPEG frame is added, it will be possible to losslessly reconstruct the JPEG codestream.
    ///
    /// If this is the last frame, [`JxlEncoderCloseInput`] or [`JxlEncoderCloseFrames`] must be called before the next
    /// [`JxlEncoderProcessOutput`] call.
    ///
    /// Note, this can only be used to add JPEG frames for lossless compression. To
    /// encode with lossy compression, the JPEG must be decoded manually and a pixel
    /// buffer added using [`JxlEncoderAddImageFrame`].
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `buffer`: bytes to read JPEG from. Owned by the caller and its contents
    ///   are copied internally.
    /// - `size`: size of buffer in bytes.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success
    /// - [`JxlEncoderStatus::Error`] on error
    pub fn JxlEncoderAddJPEGFrame(
        options: *const JxlEncoderFrameSettings,
        buffer: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;

    /// Sets the buffer to read pixels from for the next image to encode. Must call
    /// [`JxlEncoderSetBasicInfo`] before [`JxlEncoderAddImageFrame`].
    ///
    /// Currently only some data types for pixel formats are supported:
    /// - [`JxlDataType::Uint8`], with range 0..255
    /// - [`JxlDataType::Uint16`], with range 0..65535
    /// - [`JxlDataType::Float16`], with nominal range 0..1
    /// - [`JxlDataType::Float`], with nominal range 0..1
    ///
    /// Note: the sample data type in `pixel_format` is allowed to be different from
    /// what is described in the [`JxlBasicInfo`]. The type in `pixel_format`,
    /// together with an optional [`JxlBitDepth`] parameter set by [`JxlEncoderSetFrameBitDepth`]
    /// describes the format of the uncompressed pixel buffer. The `bits_per_sample` and `exponent_bits_per_sample`
    /// in the [`JxlBasicInfo`] describes what will actually be encoded in the JPEG XL
    /// codestream. For example, to encode a 12-bit image, you would set
    /// `bits_per_sample` to 12, while the input frame buffer can be in the following
    /// formats:
    /// - if pixel format is in [`JxlDataType::Uint16`] with default bit depth setting
    ///   (i.e. [`JxlBitDepthType::FromPixelFormat`]), input sample values are
    ///   rescaled to 16-bit, i.e. multiplied by 65535/4095;
    /// - if pixel format is in [`JxlDataType::Uint16`] with [`JxlBitDepthType::FromCodestream`] bit depth setting,
    ///   input sample values are provided unscaled;
    /// - if pixel format is in [`JxlDataType::Float`], input sample values are
    ///   rescaled to 0..1, i.e. multiplied by 1.0/4095.0. While it is allowed, it is
    ///   obviously not recommended to use a `pixel_format` with lower precision than
    ///   what is specified in the [`JxlBasicInfo`].
    ///
    /// We support interleaved channels as described by the [`JxlPixelFormat`]:
    /// - single-channel data, e.g. grayscale
    /// - single-channel + alpha
    /// - trichromatic, e.g. RGB
    /// - trichromatic + alpha
    ///
    /// Extra channels not handled here need to be set by [`JxlEncoderSetExtraChannelBuffer`].
    /// If the image has alpha, and alpha is not passed here, it will implicitly be
    /// set to all-opaque (an alpha value of 1.0 everywhere).
    ///
    /// The pixels are assumed to be encoded in the original profile that is set with
    /// [`JxlEncoderSetColorEncoding`] or [`JxlEncoderSetICCProfile`]. If none of
    /// these functions were used, the pixels are assumed to be nonlinear sRGB for
    /// integer data types ([`JxlDataType::Uint8`], [`JxlDataType::Uint16`]), and linear
    /// sRGB for floating point data types ([`JxlDataType::Float16`], [`JxlDataType::Float`]).
    ///
    /// Sample values in floating-point pixel formats are allowed to be outside the
    /// nominal range, e.g. to represent out-of-sRGB-gamut colors in the
    /// `uses_original_profile=false` case. They are however not allowed to be NaN or
    /// +-infinity.
    ///
    /// If this is the last frame, [`JxlEncoderCloseInput`] or [`JxlEncoderCloseFrames`] must be called before the next
    /// [`JxlEncoderProcessOutput`] call.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `pixel_format`: format for pixels. Object owned by the caller and its
    ///   contents are copied internally.
    /// - `buffer`: buffer type to input the pixel data from. Owned by the caller
    ///   and its contents are copied internally.
    /// - `size`: size of buffer in bytes. This size should match what is implied
    ///   by the frame dimensions and the pixel format.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success
    /// - [`JxlEncoderStatus::Error`] on error
    pub fn JxlEncoderAddImageFrame(
        options: *const JxlEncoderFrameSettings,
        pixel_format: *const JxlPixelFormat,
        buffer: *const c_void,
        size: usize,
    ) -> JxlEncoderStatus;

    /// Sets the output processor for the encoder. This processor determines how the
    /// encoder will handle buffering, writing, seeking (if supported), and
    /// setting a finalized position during the encoding process.
    ///
    /// This should not be used when using [`JxlEncoderProcessOutput`].
    ///
    /// # Parameters
    /// - `enc`: Encoder object.
    /// - `output_processor`: The struct containing the callbacks for managing
    ///   output.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success.
    /// - [`JxlEncoderStatus::Error`] on error.
    pub fn JxlEncoderSetOutputProcessor(
        enc: *mut JxlEncoder,
        output_processor: JxlEncoderOutputProcessor,
    ) -> JxlEncoderStatus;

    /// Flushes any buffered input in the encoder, ensuring that all available input
    /// data has been processed and written to the output.
    ///
    /// This function can only be used after [`JxlEncoderSetOutputProcessor`].
    /// Before making the last call to [`JxlEncoderFlushInput`], users should call
    /// [`JxlEncoderCloseInput`] to signal the end of input data.
    ///
    /// This should not be used when using [`JxlEncoderProcessOutput`].
    ///
    /// # Parameters
    /// - `enc`: Encoder object.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success.
    /// - [`JxlEncoderStatus::Error`] on error.
    pub fn JxlEncoderFlushInput(enc: *mut JxlEncoder) -> JxlEncoderStatus;

    /// Adds a frame to the encoder using a chunked input source.
    ///
    /// This function gives a way to encode a frame by providing pixel data in a
    /// chunked or streaming manner, which can be especially useful when dealing with
    /// large images that may not fit entirely in memory or when trying to optimize
    /// memory usage. The input data is provided through callbacks defined in the
    /// [`JxlChunkedFrameInputSource`] struct. Once the frame data has been
    /// completely retrieved, this function will flush the input and close it if it
    /// is the last frame.
    ///
    /// # Parameters
    /// - `frame_settings`: Set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `is_last_frame`: Indicates if this is the last frame.
    /// - `chunked_frame_input`: Struct providing callback methods for retrieving
    ///   pixel data in chunks.
    ///
    /// # Returns
    /// Returns a status indicating the success or failure of adding the
    /// frame.
    pub fn JxlEncoderAddChunkedFrame(
        frame_settings: *const JxlEncoderFrameSettings,
        is_last_frame: JxlBool,
        chunked_frame_input: JxlChunkedFrameInputSource,
    ) -> JxlEncoderStatus;

    /// Sets the buffer to read pixels from for an extra channel at a given index.
    /// The index must be smaller than the `num_extra_channels` in the associated
    /// [`JxlBasicInfo`]. Must call [`JxlEncoderSetExtraChannelInfo`] before
    /// [`JxlEncoderSetExtraChannelBuffer`].
    ///
    /// TODO: mention what data types in pixel formats are supported.
    ///
    /// It is required to call this function for every extra channel, except for the
    /// alpha channel if that was already set through [`JxlEncoderAddImageFrame`].
    ///
    /// # Parameters
    /// - `frame_settings`: Set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `pixel_format`: Format for pixels. Object owned by the caller and its
    ///   contents are copied internally. The `num_channels` value is ignored, since the
    ///   number of channels for an extra channel is always assumed to be one.
    /// - `buffer`: Buffer type to input the pixel data from. Owned by the caller
    ///   and its contents are copied internally.
    /// - `size`: Size of buffer in bytes. This size should match what is implied
    ///   by the frame dimensions and the pixel format.
    /// - `index`: Index of the extra channel to use.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success.
    /// - [`JxlEncoderStatus::Error`] on error.
    pub fn JxlEncoderSetExtraChannelBuffer(
        frame_settings: *const JxlEncoderFrameSettings,
        pixel_format: *const JxlPixelFormat,
        buffer: *const c_void,
        size: usize,
        index: u32,
    ) -> JxlEncoderStatus;

    /// Adds a metadata box to the file format. [`JxlEncoderProcessOutput`] must be
    /// used to effectively write the box to the output. [`JxlEncoderUseBoxes`] must
    /// be enabled before using this function.
    ///
    /// Boxes allow inserting application-specific data and metadata (Exif, XML/XMP,
    /// JUMBF and user defined boxes).
    ///
    /// The box format follows ISO BMFF and shares features and box types with other
    /// image and video formats, including the Exif, XML and JUMBF boxes. The box
    /// format for JPEG XL is specified in ISO/IEC 18181-2.
    ///
    /// Boxes in general don't contain other boxes inside, except a JUMBF superbox.
    /// Boxes follow each other sequentially and are byte-aligned. If the container
    /// format is used, the JXL stream consists of concatenated boxes.
    /// It is also possible to use a direct codestream without boxes, but in that
    /// case metadata cannot be added.
    ///
    /// Each box generally has the following byte structure in the file:
    /// - 4 bytes: box size including box header (Big endian. If set to 0, an
    ///   8-byte 64-bit size follows instead).
    /// - 4 bytes: type, e.g. "JXL " for the signature box, "jxlc" for a codestream
    ///   box.
    /// - N bytes: box contents.
    ///
    /// Only the box contents are provided to the contents argument of this function,
    /// the encoder encodes the size header itself. Most boxes are written
    /// automatically by the encoder as needed ("JXL ", "ftyp", "jxll", "jxlc",
    /// "jxlp", "jxli", "jbrd"), and this function only needs to be called to add
    /// optional metadata when encoding from pixels (using [`JxlEncoderAddImageFrame`]).
    /// When recompressing JPEG files (using [`JxlEncoderAddJPEGFrame`]),
    /// if the input JPEG contains EXIF, XMP or JUMBF
    /// metadata, the corresponding boxes are already added automatically.
    ///
    /// Box types are given by 4 characters. The following boxes can be added with
    /// this function:
    /// - "Exif": a box with EXIF metadata, can be added by libjxl users, or is
    ///   automatically added when needed for JPEG reconstruction. The contents of
    ///   this box must be prepended by a 4-byte tiff header offset, which may
    ///   be 4 zero bytes in case the tiff header follows immediately.
    ///   The EXIF metadata must be in sync with what is encoded in the JPEG XL
    ///   codestream, specifically the image orientation. While this is not
    ///   recommended in practice, in case of conflicting metadata, the JPEG XL
    ///   codestream takes precedence.
    /// - "xml ": a box with XML data, in particular XMP metadata, can be added by
    ///   libjxl users, or is automatically added when needed for JPEG reconstruction
    /// - "jumb": a JUMBF superbox, which can contain boxes with different types of
    ///   metadata inside. This box type can be added by the encoder transparently,
    ///   and other libraries to create and handle JUMBF content exist.
    /// - Application-specific boxes. Their typename should not begin with "jxl" or
    ///   "JXL" or conflict with other existing typenames, and they should be
    ///   registered with MP4RA (mp4ra.org).
    ///
    /// These boxes can be stored uncompressed or Brotli-compressed (using a "brob"
    /// box), depending on the `compress_box` parameter.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `box_type`: the box type, e.g. "Exif" for EXIF metadata, "xml " for XMP or
    ///   IPTC metadata, "jumb" for JUMBF metadata.
    /// - `contents`: the full contents of the box, for example EXIF
    ///   data. ISO BMFF box header must not be included, only the contents. Owned by
    ///   the caller and its contents are copied internally.
    /// - `size`: size of the box contents.
    /// - `compress_box`: Whether to compress this box as a "brob" box. Requires
    ///   Brotli support.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success.
    /// - [`JxlEncoderStatus::Error`] on error, such as
    ///   when using this function without [`JxlEncoderUseContainer`], or adding a box
    ///   type that would result in an invalid file format.
    pub fn JxlEncoderAddBox(
        enc: *mut JxlEncoder,
        box_type: *const JxlBoxType,
        contents: *const u8,
        size: usize,
        compress_box: JxlBool,
    ) -> JxlEncoderStatus;

    /// Indicates the intention to add metadata boxes. This allows [`JxlEncoderAddBox`] to be used.
    /// When using this function, then it is required to use [`JxlEncoderCloseBoxes`] at the end.
    ///
    /// By default the encoder assumes no metadata boxes will be added.
    ///
    /// This setting can only be set at the beginning, before encoding starts.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    pub fn JxlEncoderUseBoxes(enc: *mut JxlEncoder) -> JxlEncoderStatus;

    /// Declares that no further boxes will be added with [`JxlEncoderAddBox`].
    /// This function must be called after the last box is added so the encoder knows
    /// the stream will be finished. It is not necessary to use this function if
    /// [`JxlEncoderUseBoxes`] is not used. Further frames may still be added.
    ///
    /// Must be called between [`JxlEncoderAddBox`] of the last box
    /// and the next call to [`JxlEncoderProcessOutput`], or [`JxlEncoderProcessOutput`]
    /// won't output the last box correctly.
    ///
    /// NOTE: if you don't need to close frames and boxes at separate times, you can
    /// use [`JxlEncoderCloseInput`] instead to close both at once.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    pub fn JxlEncoderCloseBoxes(enc: *mut JxlEncoder);

    /// Declares that no frames will be added and [`JxlEncoderAddImageFrame`] and
    /// [`JxlEncoderAddJPEGFrame`] won't be called anymore. Further metadata boxes
    /// may still be added. This function or [`JxlEncoderCloseInput`] must be called
    /// after adding the last frame and the next call to
    /// [`JxlEncoderProcessOutput`], or the frame won't be properly marked as last.
    ///
    /// NOTE: if you don't need to close frames and boxes at separate times, you can
    /// use [`JxlEncoderCloseInput`] instead to close both at once.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    pub fn JxlEncoderCloseFrames(enc: *mut JxlEncoder);

    /// Closes any input to the encoder, equivalent to calling [`JxlEncoderCloseFrames`]
    /// as well as calling [`JxlEncoderCloseBoxes`] if needed. No further input of any kind
    /// may be given to the encoder, but further [`JxlEncoderProcessOutput`] calls should be
    /// done to create the final output.
    ///
    /// The requirements of both [`JxlEncoderCloseFrames`] and [`JxlEncoderCloseBoxes`] apply
    /// to this function. Either this function or the other two must be called after the final
    /// frame and/or box, and the next [`JxlEncoderProcessOutput`] call, or the codestream won't
    /// be encoded correctly.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    pub fn JxlEncoderCloseInput(enc: *mut JxlEncoder);

    /// Sets the original color encoding of the image encoded by this encoder. This
    /// is an alternative to [`JxlEncoderSetICCProfile`] and only one of these two
    /// must be used. This one sets the color encoding as a [`JxlColorEncoding`],
    /// while the other sets it as ICC binary data. Must be called after
    /// [`JxlEncoderSetBasicInfo`].
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `color`: color encoding. Object owned by the caller and its contents are
    ///   copied internally.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful.
    /// - [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderSetColorEncoding(
        enc: *mut JxlEncoder,
        color: *const JxlColorEncoding,
    ) -> JxlEncoderStatus;

    /// Sets the original color encoding of the image encoded by this encoder as an
    /// ICC color profile. This is an alternative to [`JxlEncoderSetColorEncoding`]
    /// and only one of these two must be used. This one sets the color encoding as
    /// ICC binary data, while the other defines it as a [`JxlColorEncoding`]. Must
    /// be called after [`JxlEncoderSetBasicInfo`].
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `icc_profile`: bytes of the original ICC profile
    /// - `size`: size of the `icc_profile` buffer in bytes
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful.
    /// - [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderSetICCProfile(
        enc: *mut JxlEncoder,
        icc_profile: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;

    /// Initializes a [`JxlBasicInfo`] struct to default values.
    /// For forwards-compatibility, this function has to be called before values
    /// are assigned to the struct fields.
    /// The default values correspond to an 8-bit RGB image, no alpha or any
    /// other extra channels.
    ///
    /// # Parameters
    /// - `info`: global image metadata. Object owned by the caller.
    pub fn JxlEncoderInitBasicInfo(info: *mut JxlBasicInfo);

    /// Initializes a [`JxlFrameHeader`] struct to default values.
    /// For forwards-compatibility, this function has to be called before values
    /// are assigned to the struct fields.
    /// The default values correspond to a frame with no animation duration and the
    /// 'replace' blend mode. After using this function, animation duration must
    /// be set, and for composite still, blend settings must be set.
    ///
    /// # Parameters
    /// - `frame_header`: frame metadata. Object owned by the caller.
    pub fn JxlEncoderInitFrameHeader(frame_header: *mut JxlFrameHeader);

    /// Initializes a [`JxlBlendInfo`] struct to default values.
    /// For forwards-compatibility, this function has to be called before values
    /// are assigned to the struct fields.
    ///
    /// # Parameters
    /// - `blend_info`: blending info. Object owned by the caller.
    pub fn JxlEncoderInitBlendInfo(blend_info: *mut JxlBlendInfo);

    /// Sets the global metadata of the image encoded by this encoder.
    ///
    /// If the [`JxlBasicInfo`] contains information of extra channels beyond an
    /// alpha channel, then [`JxlEncoderSetExtraChannelInfo`] must be called between
    /// [`JxlEncoderSetBasicInfo`] and [`JxlEncoderAddImageFrame`]. In order to
    /// indicate extra channels, the value of `info.num_extra_channels` should be set
    /// to the number of extra channels, also counting the alpha channel if present.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `info`: global image metadata. Object owned by the caller and its
    ///   contents are copied internally.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful.
    /// - [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderSetBasicInfo(
        enc: *mut JxlEncoder,
        info: *const JxlBasicInfo,
    ) -> JxlEncoderStatus;

    /// Sets the upsampling method the decoder will use in case there are frames
    /// with [`JxlEncoderFrameSettingsSetOption`] set. This is useful in combination
    /// with the [`JxlEncoderFrameSettingsSetOption`] option, to control
    /// the type of upsampling that will be used.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `factor`: upsampling factor to configure (1, 2, 4 or 8; for 1 this
    ///   function has no effect at all)
    /// - `mode`: upsampling mode to use for this upsampling:
    ///   - `-1`: default (good for photographic images, no signaling overhead)
    ///   - `0`: nearest neighbor (good for pixel art)
    ///   - `1`: 'pixel dots' (same as NN for 2x, diamond-shaped 'pixel dots' for 4x/8x)
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful,
    /// - [`JxlEncoderStatus::Error`] otherwise
    pub fn JxlEncoderSetUpsamplingMode(
        enc: *mut JxlEncoder,
        factor: i64,
        mode: i64,
    ) -> JxlEncoderStatus;

    /// Initializes a [`JxlExtraChannelInfo`] struct to default values.
    /// For forwards-compatibility, this function has to be called before values
    /// are assigned to the struct fields.
    /// The default values correspond to an 8-bit channel of the provided type.
    ///
    /// # Parameters
    /// - `channel_type`: type of the extra channel.
    /// - `info`: global extra channel metadata. Object owned by the caller and its
    ///   contents are copied internally.
    pub fn JxlEncoderInitExtraChannelInfo(
        channel_type: JxlExtraChannelType,
        info: *mut JxlExtraChannelInfo,
    );

    /// Sets information for the extra channel at the given index. The index
    /// must be smaller than `num_extra_channels` in the associated [`JxlBasicInfo`].
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `index`: index of the extra channel to set.
    /// - `info`: global extra channel metadata. Object owned by the caller and its
    ///   contents are copied internally.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success.
    /// - [`JxlEncoderStatus::Error`] on error.
    pub fn JxlEncoderSetExtraChannelInfo(
        enc: *mut JxlEncoder,
        index: usize,
        info: *const JxlExtraChannelInfo,
    ) -> JxlEncoderStatus;

    /// Sets the name for the extra channel at the given index in UTF-8. The index
    /// must be smaller than the `num_extra_channels` in the associated [`JxlBasicInfo`].
    ///
    /// TODO: remove size parameter for consistency with [`JxlEncoderSetFrameName`]
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `index`: index of the extra channel to set.
    /// - `name`: buffer with the name of the extra channel.
    /// - `size`: size of the name buffer in bytes, not counting the terminating
    ///   character.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] on success.
    /// - [`JxlEncoderStatus::Error`] on error.
    pub fn JxlEncoderSetExtraChannelName(
        enc: *mut JxlEncoder,
        index: usize,
        name: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;

    /// Sets a frame-specific option of integer type to the encoder options.
    /// The [`JxlEncoderFrameSettingId`] argument determines which option is set.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `option`: ID of the option to set.
    /// - `value`: Integer value to set for this option.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful.
    /// - [`JxlEncoderStatus::Error`] in case of an error, such as invalid or unknown option id, or
    ///   invalid integer value for the given option. If an error is returned, the
    ///   state of the [`JxlEncoderFrameSettings`] object is still valid and is the same as before
    ///   this function was called.
    pub fn JxlEncoderFrameSettingsSetOption(
        frame_settings: *mut JxlEncoderFrameSettings,
        option: JxlEncoderFrameSettingId,
        value: i64,
    ) -> JxlEncoderStatus;

    /// Sets a frame-specific option of float type to the encoder options.
    /// The [`JxlEncoderFrameSettingId`] argument determines which option is set.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `option`: ID of the option to set.
    /// - `value`: Float value to set for this option.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful.
    /// - [`JxlEncoderStatus::Error`] in case of an error, such as invalid or unknown option id, or
    ///   invalid float value for the given option. If an error is returned, the
    ///   state of the [`JxlEncoderFrameSettings`] object is still valid and is the same as before
    ///   this function was called.
    pub fn JxlEncoderFrameSettingsSetFloatOption(
        frame_settings: *mut JxlEncoderFrameSettings,
        option: JxlEncoderFrameSettingId,
        value: f32,
    ) -> JxlEncoderStatus;

    /// Forces the encoder to use the box-based container format (BMFF) even
    /// when not necessary.
    ///
    /// When using [`JxlEncoderUseBoxes`], [`JxlEncoderStoreJPEGMetadata`] or
    /// [`JxlEncoderSetCodestreamLevel`] with level 10, the encoder will automatically
    /// also use the container format, it is not necessary to use
    /// [`JxlEncoderUseContainer`] for those use cases.
    ///
    /// By default this setting is disabled.
    ///
    /// This setting can only be set at the beginning, before encoding starts.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `use_container`: true if the encoder should always output the JPEG XL
    ///   container format, false to only output it when necessary.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful.
    /// - [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderUseContainer(enc: *mut JxlEncoder, use_container: bool) -> JxlEncoderStatus;

    /// Configure the encoder to store JPEG reconstruction metadata in the JPEG XL
    /// container.
    ///
    /// If this is set to true and a single JPEG frame is added, it will be
    /// possible to losslessly reconstruct the JPEG codestream.
    ///
    /// This setting can only be set at the beginning, before encoding starts.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `store_jpeg_metadata`: true if the encoder should store JPEG metadata.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful.
    /// - [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderStoreJPEGMetadata(
        enc: *mut JxlEncoder,
        store_jpeg_metadata: bool,
    ) -> JxlEncoderStatus;

    /// Sets the feature level of the JPEG XL codestream. Valid values are 5 and
    /// 10, or -1 (to choose automatically). Using the minimum required level, or
    /// level 5 in most cases, is recommended for compatibility with all decoders.
    ///
    /// Level 5: for end-user image delivery, this level is the most widely
    /// supported level by image decoders and the recommended level to use unless a
    /// level 10 feature is absolutely necessary. Supports a maximum resolution
    /// 268435456 pixels total with a maximum width or height of 262144 pixels,
    /// maximum 16-bit color channel depth, maximum 120 frames per second for
    /// animation, maximum ICC color profile size of 4 MiB, it allows all color
    /// models and extra channel types except CMYK and the [`JxlExtraChannelType::Black`]
    /// extra channel, and a maximum of 4 extra channels in addition to the 3 color
    /// channels. It also sets boundaries to certain internally used coding tools.
    ///
    /// Level 10: this level removes or increases the bounds of most of the level
    /// 5 limitations, allows CMYK color and up to 32 bits per color channel, but
    /// may be less widely supported.
    ///
    /// The default value is -1. This means the encoder will automatically choose
    /// between level 5 and level 10 based on what information is inside the [`JxlBasicInfo`]
    /// structure. Do note that some level 10 features, particularly those used by animated
    /// JPEG XL codestreams, might require level 10, even though the [`JxlBasicInfo`] only
    /// suggests level 5. In this case, the level must be explicitly set to 10, otherwise
    /// the encoder will return an error. The encoder will restrict internal encoding choices
    /// to those compatible with the level setting.
    ///
    /// This setting can only be set at the beginning, before encoding starts.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `level`: the level value to set, must be -1, 5, or 10.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful.
    /// - [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderSetCodestreamLevel(enc: *mut JxlEncoder, level: i32) -> JxlEncoderStatus;

    /// Returns the codestream level required to support the currently configured
    /// settings and basic info. This function can only be used at the beginning,
    /// before encoding starts, but after setting basic info.
    ///
    /// This does not support per-frame settings, only global configuration, such as
    /// the image dimensions, that are known at the time of writing the header of
    /// the JPEG XL file.
    ///
    /// If this returns 5, nothing needs to be done and the codestream can be
    /// compatible with any decoder. If this returns 10, [`JxlEncoderSetCodestreamLevel`]
    /// has to be used to set the codestream level to 10, or the encoder can be configured
    /// differently to allow using the more compatible level 5.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    ///
    /// # Returns
    /// - `-1`: if no level can support the configuration (e.g. image dimensions
    ///   larger than even level 10 supports),
    /// - `5`: if level 5 is supported,
    /// - `10`: if setting the codestream level to 10 is required.
    pub fn JxlEncoderGetRequiredCodestreamLevel(enc: *mut JxlEncoder) -> i32;

    /// Enables lossless encoding.
    ///
    /// This is not an option like the others on itself, but rather while enabled it
    /// overrides a set of existing options (such as distance, modular mode and
    /// color transform) that enables bit-for-bit lossless encoding.
    ///
    /// When disabled, those options are not overridden, but since those options
    /// could still have been manually set to a combination that operates losslessly,
    /// using this function with lossless set to [`JxlBool::False`] does not
    /// guarantee lossy encoding, though the default set of options is lossy.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `lossless`: whether to override options for lossless mode
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful, [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderSetFrameLossless(
        frame_settings: *mut JxlEncoderFrameSettings,
        lossless: bool,
    ) -> JxlEncoderStatus;

    /// Sets the distance level for lossy compression: target max butteraugli
    /// distance, lower = higher quality. Range: 0 .. 25.
    /// 0.0 = mathematically lossless (however, use [`JxlEncoderSetFrameLossless`]
    /// instead to use true lossless, as setting distance to 0 alone is not the only
    /// requirement). 1.0 = visually lossless. Recommended range: 0.5 .. 3.0. Default
    /// value: 1.0.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `distance`: the distance value to set.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful, [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderSetFrameDistance(
        options: *mut JxlEncoderFrameSettings,
        distance: f32,
    ) -> JxlEncoderStatus;

    /// Sets the distance level for lossy compression of extra channels.
    /// The distance is as in [`JxlEncoderSetFrameDistance`] (lower = higher
    /// quality). If not set, or if set to the special value -1, the distance that
    /// was set with [`JxlEncoderSetFrameDistance`] will be used.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `index`: index of the extra channel to set a distance value for.
    /// - `distance`: the distance value to set.
    ///
    /// # Returns
    /// - [`JxlEncoderStatus::Success`] if the operation was successful, [`JxlEncoderStatus::Error`] otherwise.
    pub fn JxlEncoderSetExtraChannelDistance(
        frame_settings: *mut JxlEncoderFrameSettings,
        index: usize,
        distance: f32,
    ) -> JxlEncoderStatus;

    /// Maps JPEG-style quality factor to distance.
    ///
    /// This function takes in input a JPEG-style quality factor `quality` and
    /// produces as output a `distance` value suitable to be used with
    /// [`JxlEncoderSetFrameDistance`] and [`JxlEncoderSetExtraChannelDistance`].
    ///
    /// The `distance` value influences the level of compression, with lower values
    /// indicating higher quality:
    /// - 0.0 implies lossless compression (however, note that calling
    ///   [`JxlEncoderSetFrameLossless`] is required).
    /// - 1.0 represents a visually lossy compression, which is also the default
    ///   setting.
    ///
    /// The `quality` parameter, ranging up to 100, is inversely related to
    /// `distance`:
    /// - A `quality` of 100.0 maps to a `distance` of 0.0 (lossless).
    /// - A `quality` of 90.0 corresponds to a `distance` of 1.0.
    ///
    /// Recommended Range:
    /// - `distance`: 0.5 to 3.0.
    /// - corresponding `quality`: approximately 96 to 68.
    ///
    /// Allowed Range:
    /// - `distance`: 0.0 to 25.0.
    /// - corresponding `quality`: 100.0 to 0.0.
    ///
    /// Note: the `quality` parameter has no consistent psychovisual meaning
    /// across different codecs and libraries. Using the mapping defined by
    /// [`JxlEncoderDistanceFromQuality`] will result in a visual quality roughly
    /// equivalent to what would be obtained with `libjpeg-turbo` with the same
    /// `quality` parameter, but that is by no means guaranteed; do not assume that
    /// the same quality value will result in similar file sizes and image quality
    /// across different codecs.
    pub fn JxlEncoderDistanceFromQuality(quality: f32) -> f32;

    /// Create a new set of encoder options, with all values initially copied from
    /// the `source` options, or set to default if `source` is `NULL`.
    ///
    /// The returned pointer is an opaque struct tied to the encoder and it will be
    /// deallocated by the encoder when [`JxlEncoderDestroy`] is called. For
    /// functions taking both a [`JxlEncoder`] and a [`JxlEncoderFrameSettings`],
    /// only [`JxlEncoderFrameSettings`] created with this function for the same
    /// encoder instance can be used.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    /// - `source`: source options to copy initial values from, or `NULL` to get
    ///   defaults initialized to defaults.
    ///
    /// # Returns
    /// The opaque struct pointer identifying a new set of encoder options.
    pub fn JxlEncoderFrameSettingsCreate(
        enc: *mut JxlEncoder,
        source: *const JxlEncoderFrameSettings,
    ) -> *mut JxlEncoderFrameSettings;

    /// Sets a color encoding to be sRGB.
    ///
    /// # Parameters
    /// - `color_encoding`: color encoding instance.
    /// - `is_gray`: whether the color encoding should be gray scale or color.
    pub fn JxlColorEncodingSetToSRGB(color_encoding: *mut JxlColorEncoding, is_gray: bool);

    /// Sets a color encoding to be linear sRGB.
    ///
    /// # Parameters
    /// - `color_encoding`: [color encoding instance](JxlColorEncoding).
    /// - `is_gray`: whether the color encoding should be gray scale or color.
    pub fn JxlColorEncodingSetToLinearSRGB(color_encoding: *mut JxlColorEncoding, is_gray: bool);

    /// Enables usage of expert options.
    ///
    /// At the moment, the only expert option is setting an effort value of 11,
    /// which gives the best compression for pixel-lossless modes but is very slow.
    ///
    /// # Parameters
    /// - `enc`: encoder object.
    pub fn JxlEncoderAllowExpertOptions(enc: *mut JxlEncoder);

    /// Sets the given debug image callback that will be used by the encoder to
    /// output various debug images during encoding.
    ///
    /// This only has any effect if the encoder was compiled with the appropriate
    /// debug build flags.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `callback`: used to return the debug image.
    /// - `opaque`: user supplied parameter to the image callback.
    pub fn JxlEncoderSetDebugImageCallback(
        frame_settings: *mut JxlEncoderFrameSettings,
        callback: JxlDebugImageCallback,
        opaque: *mut c_void,
    );

    /// Sets the given stats object for gathering various statistics during encoding.
    ///
    /// This only has any effect if the encoder was compiled with the appropriate
    /// debug build flags.
    ///
    /// # Parameters
    /// - `frame_settings`: set of options and metadata for this frame. Also
    ///   includes reference to the encoder object.
    /// - `stats`: object that can be used to query the gathered stats (created
    ///   by [`JxlEncoderStatsCreate`])
    pub fn JxlEncoderCollectStats(
        frame_settings: *mut JxlEncoderFrameSettings,
        stats: *mut JxlEncoderStats,
    );
}
