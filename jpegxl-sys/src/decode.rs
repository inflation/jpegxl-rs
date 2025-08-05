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

//! Decoding API for JPEG XL.

use std::{
    ffi::c_void,
    os::raw::{c_char, c_int},
};

use crate::{
    color::{cms_interface::JxlCmsInterface, color_encoding::JxlColorEncoding},
    common::memory_manager::JxlMemoryManager,
    common::types::{JxlBitDepth, JxlBool, JxlBoxType, JxlPixelFormat},
    metadata::codestream_header::{
        JxlBasicInfo, JxlBlendInfo, JxlExtraChannelInfo, JxlFrameHeader,
    },
    threads::parallel_runner::JxlParallelRunner,
};
#[cfg(doc)]
use crate::{
    common::types::JxlBitDepthType,
    metadata::codestream_header::{JxlOrientation, JxlPreviewHeader},
};

/// The result of [`JxlSignatureCheck`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlSignature {
    /// Not enough bytes were passed to determine if a valid signature was found.
    NotEnoughBytes = 0,
    /// No valid JPEG XL header was found.
    Invalid = 1,
    /// A valid JPEG XL codestream signature was found, that is a JPEG XL image
    /// without container.
    Codestream = 2,
    /// A valid container signature was found, that is a JPEG XL image embedded
    /// in a box format container.
    Container = 3,
}

/// Opaque structure that holds the JPEG XL decoder.
///
/// Allocated and initialized with [`JxlDecoderCreate`].
/// Cleaned up and deallocated with [`JxlDecoderDestroy`].
#[repr(C)]
pub struct JxlDecoder {
    _unused: [u8; 0],
}

/// Return value for [`JxlDecoderProcessInput`].
/// The values from [`JxlDecoderStatus::BasicInfo`] onwards are optional informative
/// events that can be subscribed to, they are never returned if they
/// have not been registered with [`JxlDecoderSubscribeEvents`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlDecoderStatus {
    /// Function call finished successfully, or decoding is finished and there is
    /// nothing more to be done.
    ///
    /// Note that [`JxlDecoderProcessInput`] will return [`JxlDecoderStatus::Success`] if
    /// all events that were registered with [`JxlDecoderSubscribeEvents`] were
    /// processed, even before the end of the JPEG XL codestream.
    ///
    /// In this case, the return value [`JxlDecoderReleaseInput`] will be the same
    /// as it was at the last signaled event. E.g. if [`JxlDecoderStatus::FullImage`] was
    /// subscribed to, then all bytes from the end of the JPEG XL codestream
    /// (including possible boxes needed for jpeg reconstruction) will be returned
    /// as unprocessed.
    Success = 0,

    /// An error occurred, for example invalid input file or out of memory.
    /// TODO: add function to get error information from decoder.
    Error = 1,

    /// The decoder needs more input bytes to continue. Before the next [`JxlDecoderProcessInput`]
    /// call, more input data must be set, by calling [`JxlDecoderReleaseInput`] (if input was set previously)
    /// and then calling [`JxlDecoderSetInput`]. [`JxlDecoderReleaseInput`] returns how many bytes
    /// are not yet processed, before a next call to [`JxlDecoderProcessInput`]
    /// all unprocessed bytes must be provided again (the address need not match,
    /// but the contents must), and more bytes must be concatenated after the
    /// unprocessed bytes.
    /// In most cases, [`JxlDecoderReleaseInput`] will return no unprocessed bytes
    /// at this event, the only exceptions are if the previously set input ended
    /// within (a) the raw codestream signature, (b) the signature box, (c) a box
    /// header, or (d) the first 4 bytes of a `brob`, `ftyp`, or `jxlp` box. In any
    /// of these cases the number of unprocessed bytes is less than 20.
    NeedMoreInput = 2,

    /// The decoder is able to decode a preview image and requests setting a
    /// preview output buffer using [`JxlDecoderSetPreviewOutBuffer`]. This occurs
    /// if [`JxlDecoderStatus::PreviewImage`] is requested and it is possible to decode a
    /// preview image from the codestream and the preview out buffer was not yet
    /// set. There is maximum one preview image in a codestream.
    /// In this case, [`JxlDecoderReleaseInput`] will return all bytes from the
    /// end of the frame header (including `ToC`) of the preview frame as
    /// unprocessed.
    NeedPreviewOutBuffer = 3,

    /// The decoder requests an output buffer to store the full resolution image,
    /// which can be set with [`JxlDecoderSetImageOutBuffer`] or with [`JxlDecoderSetImageOutCallback`].
    /// This event re-occurs for new frames if there are multiple animation frames and requires setting an output again.
    /// In this case, [`JxlDecoderReleaseInput`] will return all bytes from the
    /// end of the frame header (including `ToC`) as unprocessed.
    NeedImageOutBuffer = 5,

    /// The JPEG reconstruction buffer is too small for reconstructed JPEG
    /// codestream to fit. [`JxlDecoderSetJPEGBuffer`] must be called again to
    /// make room for remaining bytes. This event may occur multiple times
    /// after [`JxlDecoderStatus::JPEGReconstruction`].
    JPEGNeedMoreOutput = 6,

    /// The box contents output buffer is too small. [`JxlDecoderSetBoxBuffer`]
    /// must be called again to make room for remaining bytes. This event may occur
    /// multiple times after [`JxlDecoderStatus::Box`].
    BoxNeedMoreOutput = 7,

    /// Informative event by [`JxlDecoderProcessInput`]: Basic information such as image dimensions and
    /// extra channels. This event occurs max once per image.
    /// In this case, [`JxlDecoderReleaseInput`] will return all bytes from the
    /// end of the basic info as unprocessed (including the last byte of basic info
    /// if it did not end on a byte boundary).
    BasicInfo = 0x40,

    /// Informative event by [`JxlDecoderProcessInput`]: Color encoding or ICC profile from the
    /// codestream header. This event occurs max once per image and always later
    /// than [`JxlDecoderStatus::BasicInfo`] and earlier than any pixel data.
    /// In this case, [`JxlDecoderReleaseInput`] will return all bytes from the
    /// end of the image header (which is the start of the first frame) as
    /// unprocessed.
    ColorEncoding = 0x100,

    /// Informative event by [`JxlDecoderProcessInput`]: Preview image, a small frame, decoded. This
    /// event can only happen if the image has a preview frame encoded. This event
    /// occurs max once for the codestream and always later than [`JxlDecoderStatus::ColorEncoding`]
    /// and before [`JxlDecoderStatus::Frame`]. In this case, [`JxlDecoderReleaseInput`] will return all bytes
    /// from the end of the preview frame as unprocessed.
    PreviewImage = 0x200,

    /// Informative event by [`JxlDecoderProcessInput`]: Beginning of a frame. [`JxlDecoderGetFrameHeader`] can be
    /// used at this point.
    ///
    /// ## Note:
    ///
    /// a JPEG XL image can have internal frames that are not intended to be
    /// displayed (e.g. used for compositing a final frame), but this only returns
    /// displayed frames, unless [`JxlDecoderSetCoalescing`] was set to [`JxlBool::False`]:
    /// in that case, the individual layers are returned, without blending.
    /// Note that even when coalescing is disabled, only frames of type `kRegularFrame` are returned;
    /// frames of type `kReferenceOnly` and `kLfFrame` are always for internal purposes only and cannot be accessed.
    /// A displayed frame either has an animation duration or is the only or last
    /// frame in the image. This event occurs max once per displayed frame, always
    /// later than [`JxlDecoderStatus::ColorEncoding`], and always earlier than any pixel
    /// data. While JPEG XL supports encoding a single frame as the composition of
    /// multiple internal sub-frames also called frames, this event is not
    /// indicated for the internal frames. In this case, [`JxlDecoderReleaseInput`] will return all bytes
    /// from the end of the frame header (including `ToC`) as unprocessed.
    Frame = 0x400,

    /// Informative event by [`JxlDecoderProcessInput`]: full frame (or layer, in case coalescing is
    /// disabled) is decoded. [`JxlDecoderSetImageOutBuffer`] must be used after
    /// getting the basic image information to be able to get the image pixels, if
    /// not this return status only indicates we're past this point in the
    /// codestream. This event occurs max once per frame.
    /// In this case, [`JxlDecoderReleaseInput`] will return all bytes from the
    /// end of the frame (or if [`JxlDecoderStatus::JPEGReconstruction`] is subscribed to,
    /// from the end of the last box that is needed for jpeg reconstruction) as
    /// unprocessed.
    FullImage = 0x1000,

    /// Informative event by [`JxlDecoderProcessInput`]: JPEG reconstruction data decoded.
    /// [`JxlDecoderSetJPEGBuffer`] may be used to set a JPEG reconstruction buffer
    /// after getting the JPEG reconstruction data. If a JPEG reconstruction buffer
    /// is set a byte stream identical to the JPEG codestream used to encode the
    /// image will be written to the JPEG reconstruction buffer instead of pixels
    /// to the image out buffer. This event occurs max once per image and always
    /// before [`JxlDecoderStatus::FullImage`].
    /// In this case, [`JxlDecoderReleaseInput`] will return all bytes from the
    /// end of the `jbrd` box as unprocessed.
    JPEGReconstruction = 0x2000,

    /// Informative event by [`JxlDecoderProcessInput`]: The header of a box of the container format
    /// (BMFF) is decoded. The following API functions related to boxes can be used
    /// after this event:
    ///  - [`JxlDecoderSetBoxBuffer`] and [`JxlDecoderReleaseBoxBuffer`]: set and release a buffer to get the box
    ///    data.
    ///  - [`JxlDecoderGetBoxType`] get the 4-character box typename.
    ///  - [`JxlDecoderGetBoxSizeRaw`] get the size of the box as it appears in
    ///    the container file, not decompressed.
    ///  - [`JxlDecoderSetDecompressBoxes`] to configure whether to get the box
    ///    data decompressed, or possibly compressed.
    ///
    /// Boxes can be compressed. This is so when their box type is
    /// "brob". In that case, they have an underlying decompressed box
    /// type and decompressed data. [`JxlDecoderSetDecompressBoxes`] allows
    /// configuring which data to get. Decompressing requires
    /// Brotli. [`JxlDecoderGetBoxType`] has a flag to get the compressed box
    /// type, which can be "brob", or the decompressed box type. If a box
    /// is not compressed (its compressed type is not "brob"), then
    /// the output decompressed box type and data is independent of what
    /// setting is configured.
    ///
    /// The buffer set with [`JxlDecoderSetBoxBuffer`] must be set again for each
    /// next box to be obtained, or can be left unset to skip outputting this box.
    /// The output buffer contains the full box data when the
    /// [`JxlDecoderStatus::BoxComplete`] (if subscribed to) or subsequent [`JxlDecoderStatus::Success`]
    /// or subsequent [`JxlDecoderStatus::Box`] event occurs. [`JxlDecoderStatus::Box`] occurs for all boxes,
    /// including non-metadata boxes such as the signature box or codestream boxes.
    /// To check whether the box is a metadata type for respectively EXIF, XMP or
    /// JUMBF, use [`JxlDecoderGetBoxType`] and check for types "Exif", "xml " and
    /// "jumb" respectively.
    ///
    /// In this case, [`JxlDecoderReleaseInput`] will return all bytes from the
    /// start of the box header as unprocessed.
    Box = 0x4000,

    /// Informative event by [`JxlDecoderProcessInput`]: a progressive step in decoding the frame is
    /// reached. When calling [`JxlDecoderFlushImage`] at this point, the flushed
    /// image will correspond exactly to this point in decoding, and not yet
    /// contain partial results (such as partially more fine detail) of a next
    /// step. By default, this event will trigger maximum once per frame, when a
    /// 8x8th resolution (DC) image is ready (the image data is still returned at
    /// full resolution, giving upscaled DC). Use [`JxlDecoderSetProgressiveDetail`] to configure more fine-grainedness.
    /// The event is not guaranteed to trigger, not all images have progressive steps
    /// or DC encoded.
    /// In this case, [`JxlDecoderReleaseInput`] will return all bytes from the
    /// end of the section that was needed to produce this progressive event as
    /// unprocessed.
    FrameProgression = 0x8000,

    /// The box being decoded is now complete. This is only emitted if a buffer
    /// was set for the box.
    BoxComplete = 0x10000,
}

/// Types of progressive detail.
/// Setting a progressive detail with value N implies all progressive details
/// with smaller or equal value. Currently only the following level of
/// progressive detail is implemented:
///  - [`JxlProgressiveDetail::DC`] (which implies [`JxlProgressiveDetail::Frames`])
///  - [`JxlProgressiveDetail::LastPasses`] (which implies [`JxlProgressiveDetail::DC`]
///    and [`JxlProgressiveDetail::Frames`])
///  - [`JxlProgressiveDetail::Passes`] (which implies [`JxlProgressiveDetail::LastPasses`],
///    [`JxlProgressiveDetail::DC`] and [`JxlProgressiveDetail::Frames`])
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlProgressiveDetail {
    /// After completed kRegularFrames
    Frames = 0,
    /// After completed DC (1:8)
    DC = 1,
    /// After completed AC passes that are the last pass for their resolution target.
    LastPasses = 2,
    /// After completed AC passes that are not the last pass for their resolution target.
    Passes = 3,
    /// During DC frame when lower resolutions are completed (1:32, 1:16)
    DCProgressive = 4,
    /// After completed groups
    DCGroups = 5,
    /// After completed groups
    Groups = 6,
}

/// Defines which color profile to get: the profile from the codestream
/// metadata header, which represents the color profile of the original image,
/// or the color profile from the pixel data produced by the decoder. Both are
/// the same if the [`JxlBasicInfo`] has `uses_original_profile` set.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlColorProfileTarget {
    Original = 0,
    Data = 1,
}

/// Function type for [`JxlDecoderSetImageOutCallback`].
///
/// The callback may be called simultaneously by different threads when using a
/// threaded parallel runner, on different pixels.
///
/// # Parameters
/// - `opaque`: optional user data, as given to [`JxlDecoderSetImageOutCallback`].
/// - `x`: horizontal position of leftmost pixel of the pixel data.
/// - `y`: vertical position of the pixel data.
/// - `num_pixels`: amount of pixels included in the pixel data, horizontally.
///   This is not the same as xsize of the full image, it may be smaller.
/// - `pixels`: pixel data as a horizontal stripe, in the format passed to
///   [`JxlDecoderSetImageOutCallback`]. The memory is not owned by the user, and
///   is only valid during the time the callback is running.
pub type JxlImageOutCallback = extern "C" fn(
    opaque: *mut c_void,
    x: usize,
    y: usize,
    num_pixels: usize,
    pixels: *const c_void,
);

/// Initialization callback for [`JxlDecoderSetMultithreadedImageOutCallback`].
///
/// # Parameters
/// - `init_opaque`: optional user data, as given to
///   [`JxlDecoderSetMultithreadedImageOutCallback`].
/// - `num_threads`: maximum number of threads that will call the [`run`](JxlImageOutRunCallback)
///   callback concurrently.
/// - `num_pixels_per_thread`: maximum number of pixels that will be passed in
///   one call to [`run`](JxlImageOutRunCallback).
///
/// # Returns
/// - a pointer to data that will be passed to the [`run`](JxlImageOutRunCallback) callback, or
///   `NULL` if initialization failed.
pub type JxlImageOutInitCallback = extern "C" fn(
    init_opaque: *mut c_void,
    num_threads: usize,
    num_pixels_per_thread: usize,
) -> *mut c_void;

/// Worker callback for [`JxlDecoderSetMultithreadedImageOutCallback`]
///
/// # Parameters
/// - `run_opaque`: user data returned by the [`init`](JxlImageOutInitCallback) callback.
/// - `thread_id`: number in `[0, num_threads)` identifying the thread of the
///   current invocation of the callback.
/// - `x`: horizontal position of the first (leftmost) pixel of the pixel data.
/// - `y`: vertical position of the pixel data.
/// - `num_pixels`: number of pixels in the pixel data. May be less than the
///   full `xsize` of the image, and will be at most equal to the `num_pixels_per_thread`
///   that was passed to [`init`](JxlImageOutInitCallback).
/// - `pixels`: pixel data as a horizontal stripe, in the format passed to
///   [`JxlDecoderSetMultithreadedImageOutCallback`]. The data pointed to
///   remains owned by the caller and is only guaranteed to outlive the current
///   callback invocation.
pub type JxlImageOutRunCallback = extern "C" fn(
    run_opaque: *mut c_void,
    thread_id: usize,
    x: usize,
    y: usize,
    num_pixels: usize,
    pixels: *const c_void,
);

///
/// Destruction callback for [`JxlDecoderSetMultithreadedImageOutCallback`],
/// called after all invocations of the [`run`](JxlImageOutCallback) callback to perform any
/// appropriate clean-up of the `run_opaque` data returned by [`init`](JxlImageOutInitCallback).
///
/// # Parameters
/// - `run_opaque`: user data returned by the [`init`](JxlImageOutInitCallback) callback.
pub type JxlImageOutDestroyCallback = extern "C" fn(run_opaque: *mut c_void);

extern "C-unwind" {
    /// Decoder library version.
    ///
    /// # Returns
    /// The decoder library version as an integer:
    /// `MAJOR_VERSION * 1000000 + MINOR_VERSION * 1000 + PATCH_VERSION`. For example,
    /// version 1.2.3 would return 1002003.
    pub fn JxlDecoderVersion() -> u32;

    /// JPEG XL signature identification.
    ///
    /// Checks if the passed buffer contains a valid JPEG XL signature. The passed `buf` of size
    /// `size` doesn't need to be a full image, only the beginning of the file.
    ///
    /// # Returns
    /// A flag indicating if a JPEG XL signature was found and what type.
    /// - [`JxlSignature::NotEnoughBytes`] if not enough bytes were passed to
    ///   determine if a valid signature is there.
    /// - [`JxlSignature::Invalid`] if no valid signature found for JPEG XL decoding.
    /// - [`JxlSignature::Codestream`] if a valid JPEG XL codestream signature was
    ///   found.
    /// - [`JxlSignature::Container`] if a valid JPEG XL container signature was found.
    pub fn JxlSignatureCheck(buf: *const u8, len: usize) -> JxlSignature;

    /// Creates an instance of [`JxlDecoder`] and initializes it.
    ///
    /// `memory_manager` will be used for all the library dynamic allocations made
    /// from this instance. The parameter may be `NULL`, in which case the default
    /// allocator will be used. See [`crate::common::memory_manager`] for details.
    ///
    /// # Parameters
    /// - `memory_manager`: custom allocator function. It may be `NULL`. The memory
    ///   manager will be copied internally.
    ///
    /// # Returns
    /// - `NULL` if the instance cannot be allocated or initialized.
    /// - Pointer to initialized [`JxlDecoder`] otherwise.
    pub fn JxlDecoderCreate(memory_manager: *const JxlMemoryManager) -> *mut JxlDecoder;

    /// Re-initializes a [`JxlDecoder`] instance, so it can be re-used for decoding
    /// another image. All state and settings are reset as if the object was
    /// newly created with [`JxlDecoderCreate`], but the memory manager is kept.
    ///
    /// # Parameters
    /// - `dec`: instance to be re-initialized.
    pub fn JxlDecoderReset(dec: *mut JxlDecoder);

    /// Deinitializes and frees [`JxlDecoder`] instance.
    ///
    /// # Parameters
    /// - `dec`: instance to be cleaned up and deallocated.
    pub fn JxlDecoderDestroy(dec: *mut JxlDecoder);

    /// Rewinds decoder to the beginning. The same input must be given again from
    /// the beginning of the file and the decoder will emit events from the beginning
    /// again. When rewinding (as opposed to [`JxlDecoderReset`]), the decoder can
    /// keep state about the image, which it can use to skip to a requested frame
    /// more efficiently with [`JxlDecoderSkipFrames`]. Settings such as parallel
    /// runner or subscribed events are kept. After rewind, [`JxlDecoderSubscribeEvents`]
    /// can be used again, and it is feasible to leave out events that were already
    /// handled before, such as [`JxlDecoderStatus::BasicInfo`] and [`JxlDecoderStatus::ColorEncoding`],
    /// since they will provide the same information as before.
    /// The difference to [`JxlDecoderReset`] is that some state is kept, namely
    /// settings set by a call to
    ///  - [`JxlDecoderSetCoalescing`],
    ///  - [`JxlDecoderSetDesiredIntensityTarget`],
    ///  - [`JxlDecoderSetDecompressBoxes`],
    ///  - [`JxlDecoderSetKeepOrientation`],
    ///  - [`JxlDecoderSetUnpremultiplyAlpha`],
    ///  - [`JxlDecoderSetParallelRunner`],
    ///  - [`JxlDecoderSetRenderSpotcolors`], and
    ///  - [`JxlDecoderSubscribeEvents`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    pub fn JxlDecoderRewind(dec: *mut JxlDecoder);

    /// Makes the decoder skip the next `amount` frames. It still needs to process
    /// the input, but will not output the frame events. It can be more efficient
    /// when skipping frames, and even more so when using this after [`JxlDecoderRewind`].
    /// If the decoder is already processing a frame (could have emitted [`JxlDecoderStatus::Frame`]
    /// but not yet [`JxlDecoderStatus::FullImage`]), it starts skipping from the next frame.
    /// If the amount is larger than the amount of frames remaining in the image, all remaining
    /// frames are skipped. Calling this function multiple times adds the amount to skip to the
    /// already existing amount.
    ///
    /// A frame here is defined as a frame that without skipping emits events such
    /// as [`JxlDecoderStatus::Frame`] and [`JxlDecoderStatus::FullImage`], frames that are internal
    /// to the file format but are not rendered as part of an animation, or are not
    /// the final still frame of a still image, are not counted.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `amount`: the amount of frames to skip
    pub fn JxlDecoderSkipFrames(dec: *mut JxlDecoder, amount: usize);

    /// Skips processing the current frame. Can be called after frame processing
    /// already started, signaled by a [`JxlDecoderStatus::NeedImageOutBuffer`] event,
    /// but before the corresponding [`JxlDecoderStatus::FullImage`] event. The next signaled
    /// event will be another [`JxlDecoderStatus::Frame`], or [`JxlDecoderStatus::Success`] if there
    /// are no more frames. If pixel data is required from the already processed part
    /// of the frame, [`JxlDecoderFlushImage`] must be called before this.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if there is a frame to skip, and
    /// - [`JxlDecoderStatus::Error`] if the function was not called during frame processing.
    pub fn JxlDecoderSkipCurrentFrame(dec: *mut JxlDecoder) -> JxlDecoderStatus;

    /// Set the parallel runner for multithreading. May only be set before starting
    /// decoding.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `parallel_runner`: function pointer to runner for multithreading. It may
    ///   be `NULL` to use the default, single-threaded, runner. A multithreaded
    ///   runner should be set to reach fast performance.
    /// - `parallel_runner_opaque`: opaque pointer for `parallel_runner`.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the runner was set, [`JxlDecoderStatus::Error`]
    ///   otherwise (the previous runner remains set).
    pub fn JxlDecoderSetParallelRunner(
        dec: *mut JxlDecoder,
        parallel_runner: JxlParallelRunner,
        parallel_runner_opaque: *mut c_void,
    ) -> JxlDecoderStatus;

    /// Returns a hint indicating how many more bytes the decoder is expected to
    /// need to make [`JxlDecoderGetBasicInfo`] available after the next
    /// [`JxlDecoderProcessInput`] call. This is a suggested large enough value for
    /// the amount of bytes to provide in the next [`JxlDecoderSetInput`] call, but
    /// it is not guaranteed to be an upper bound nor a lower bound. This number does
    /// not include bytes that have already been released from the input. Can be used
    /// before the first [`JxlDecoderProcessInput`] call, and is correct the first
    /// time in most cases. If not, [`JxlDecoderSizeHintBasicInfo`] can be called
    /// again to get an updated hint.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    ///
    /// # Returns
    /// - the size hint in bytes if the basic info is not yet fully decoded.
    /// - 0 when the basic info is already available.
    pub fn JxlDecoderSizeHintBasicInfo(dec: *const JxlDecoder) -> usize;

    /// Select for which informative events, i.e. [`JxlDecoderStatus::BasicInfo`], etc., the
    /// decoder should return with a status. It is not required to subscribe to any
    /// events, data can still be requested from the decoder as soon as it is available.
    /// By default, the decoder is subscribed to no events (`events_wanted == 0`), and
    /// the decoder will then only return when it cannot continue because it needs
    /// more input data or more output buffer. This function may only be called
    /// before using [`JxlDecoderProcessInput`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `events_wanted`: bitfield of desired events.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if no error, [`JxlDecoderStatus::Error`] otherwise.
    pub fn JxlDecoderSubscribeEvents(
        dec: *mut JxlDecoder,
        events_wanted: c_int,
    ) -> JxlDecoderStatus;

    /// Enables or disables preserving of as-in-bitstream pixeldata
    /// orientation. Some images are encoded with an Orientation tag
    /// indicating that the decoder must perform a rotation and/or
    /// mirroring to the encoded image data.
    ///
    /// - If `skip_reorientation` is [`JxlBool::False`] (the default): the decoder
    ///   will apply the transformation from the orientation setting, hence
    ///   rendering the image according to its specified intent. When
    ///   producing a [`JxlBasicInfo`], the decoder will always set the
    ///   orientation field to [`JxlOrientation::Identity`] (matching the returned
    ///   pixel data) and also align xsize and ysize so that they correspond
    ///   to the width and the height of the returned pixel data.
    /// - If `skip_reorientation` is [`JxlBool::True`]: the decoder will skip
    ///   applying the transformation from the orientation setting, returning
    ///   the image in the as-in-bitstream pixeldata orientation.
    ///   This may be faster to decode since the decoder doesn't have to apply the
    ///   transformation, but can cause wrong display of the image if the
    ///   orientation tag is not correctly taken into account by the user.
    ///
    /// By default, this option is disabled, and the returned pixel data is
    /// re-oriented according to the image's Orientation setting.
    ///
    /// This function must be called at the beginning, before decoding is performed.
    ///
    /// See [`JxlBasicInfo`] for the orientation field, and [`JxlOrientation`] for the
    /// possible values.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `skip_reorientation`: [`JxlBool::True`] to enable, [`JxlBool::False`] to disable.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if no error, [`JxlDecoderStatus::Error`] otherwise.
    pub fn JxlDecoderSetKeepOrientation(
        dec: *mut JxlDecoder,
        keep_orientation: JxlBool,
    ) -> JxlDecoderStatus;

    /// Enables or disables preserving of associated alpha channels. If
    /// `unpremul_alpha` is set to [`JxlBool::False`] then for associated alpha channel,
    /// the pixel data is returned with premultiplied colors. If it is set to [`JxlBool::True`],
    /// the colors will be unpremultiplied based on the alpha channel. This
    /// function has no effect if the image does not have an associated alpha
    /// channel.
    ///
    /// By default, this option is disabled, and the returned pixel data is "as is".
    ///
    /// This function must be called at the beginning, before decoding is performed.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `unpremul_alpha`: [`JxlBool::True`] to enable, [`JxlBool::False`] to disable.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if no error, [`JxlDecoderStatus::Error`] otherwise.
    pub fn JxlDecoderSetUnpremultiplyAlpha(
        dec: *mut JxlDecoder,
        unpremul_alpha: JxlBool,
    ) -> JxlDecoderStatus;

    /// Enables or disables rendering spot colors. By default, spot colors
    /// are rendered, which is OK for viewing the decoded image. If `render_spotcolors`
    /// is [`JxlBool::False`], then spot colors are not rendered, and have to be
    /// retrieved separately using [`JxlDecoderSetExtraChannelBuffer`]. This is
    /// useful for e.g. printing applications.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `render_spotcolors`: [`JxlBool::True`] to enable (default), [`JxlBool::False`] to disable.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if no error, [`JxlDecoderStatus::Error`] otherwise.
    pub fn JxlDecoderSetRenderSpotcolors(
        dec: *mut JxlDecoder,
        render_spotcolors: JxlBool,
    ) -> JxlDecoderStatus;

    /// Enables or disables coalescing of zero-duration frames. By default, frames
    /// are returned with coalescing enabled, i.e. all frames have the image
    /// dimensions, and are blended if needed. When coalescing is disabled, frames
    /// can have arbitrary dimensions, a non-zero crop offset, and blending is not
    /// performed. For display, coalescing is recommended. For loading a multi-layer
    /// still image as separate layers (as opposed to the merged image), coalescing
    /// has to be disabled.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `coalescing`: [`JxlBool::True`] to enable coalescing (default), [`JxlBool::False`] to
    ///   disable it.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if no error, [`JxlDecoderStatus::Error`] otherwise.
    pub fn JxlDecoderSetCoalescing(dec: *mut JxlDecoder, coalescing: JxlBool) -> JxlDecoderStatus;

    /// Decodes JPEG XL file using the available bytes. Requires input has been
    /// set with [`JxlDecoderSetInput`]. After [`JxlDecoderProcessInput`], input
    /// can optionally be released with [`JxlDecoderReleaseInput`] and then set
    /// again to next bytes in the stream. [`JxlDecoderReleaseInput`] returns how
    /// many bytes are not yet processed, before a next call to [`JxlDecoderProcessInput`]
    /// all unprocessed bytes must be provided again (the address need not match, but the contents must),
    /// and more bytes may be concatenated after the unprocessed bytes.
    ///
    /// The returned status indicates whether the decoder needs more input bytes, or
    /// more output buffer for a certain type of output data. No matter what the
    /// returned status is (other than [`JxlDecoderStatus::Error`]), new information,
    /// such as [`JxlDecoderGetBasicInfo`], may have become available after this call.
    /// When the return value is not [`JxlDecoderStatus::Error`] or [`JxlDecoderStatus::Success`], the
    /// decoding requires more [`JxlDecoderProcessInput`] calls to continue.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] when decoding finished and all events handled.
    ///   If you still have more unprocessed input data anyway, then you can still
    ///   continue by using [`JxlDecoderSetInput`] and calling [`JxlDecoderProcessInput`] again,
    ///   similar to handling [`JxlDecoderStatus::NeedMoreInput`]. [`JxlDecoderStatus::Success`] can occur instead of
    ///   [`JxlDecoderStatus::NeedMoreInput`] when, for example, the input data ended right at
    ///   the boundary of a box of the container format, all essential codestream
    ///   boxes were already decoded, but extra metadata boxes are still present in
    ///   the next data. [`JxlDecoderProcessInput`] cannot return success if all
    ///   codestream boxes have not been seen yet.
    /// - [`JxlDecoderStatus::Error`] when decoding failed, e.g. invalid codestream.
    /// - [`JxlDecoderStatus::NeedMoreInput`] when more input data is necessary.
    /// - [`JxlDecoderStatus::BasicInfo`] when basic info such as image dimensions is
    ///   available and this informative event is subscribed to.
    /// - [`JxlDecoderStatus::ColorEncoding`] when color profile information is
    ///   available and this informative event is subscribed to.
    /// - [`JxlDecoderStatus::PreviewImage`] when preview pixel information is
    ///   available and output in the preview buffer.
    /// - [`JxlDecoderStatus::FullImage`] when all pixel information at highest detail
    ///   is available and has been output in the pixel buffer.
    pub fn JxlDecoderProcessInput(dec: *mut JxlDecoder) -> JxlDecoderStatus;

    /// Sets input data for [`JxlDecoderProcessInput`]. The data is owned by the
    /// caller and may be used by the decoder until [`JxlDecoderReleaseInput`] is
    /// called or the decoder is destroyed or reset, so it must be kept alive until then.
    /// Cannot be called if [`JxlDecoderSetInput`] was already called and [`JxlDecoderReleaseInput`]
    /// was not yet called, and cannot be called after [`JxlDecoderCloseInput`] indicating the end
    /// of input was called.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `data`: pointer to next bytes to read from
    /// - `size`: amount of bytes available starting from data
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Error`] if input was already set without releasing or [`JxlDecoderCloseInput`]
    ///   was already called
    /// - [`JxlDecoderStatus::Success`] otherwise
    pub fn JxlDecoderSetInput(
        dec: *mut JxlDecoder,
        data: *const u8,
        size: usize,
    ) -> JxlDecoderStatus;

    /// Releases input which was provided with [`JxlDecoderSetInput`]. Between
    /// [`JxlDecoderProcessInput`] and [`JxlDecoderReleaseInput`], the user may not
    /// alter the data in the buffer. Calling [`JxlDecoderReleaseInput`] is required
    /// whenever any input is already set and new input needs to be added with
    /// [`JxlDecoderSetInput`], but is not required before [`JxlDecoderDestroy`] or
    /// [`JxlDecoderReset`]. Calling [`JxlDecoderReleaseInput`] when no input is set
    /// is not an error and returns `0`.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    ///
    /// # Returns
    /// The amount of bytes the decoder has not yet processed that are still
    /// remaining in the data set by [`JxlDecoderSetInput`], or `0` if no input is
    /// set or [`JxlDecoderReleaseInput`] was already called. For a next call to
    /// [`JxlDecoderProcessInput`], the buffer must start with these unprocessed
    /// bytes. From this value it is possible to infer the position of certain JPEG
    /// XL codestream elements (e.g. end of headers, frame start/end). See the
    /// documentation of individual values of [`JxlDecoderStatus`] for more
    /// information.
    pub fn JxlDecoderReleaseInput(dec: *mut JxlDecoder) -> usize;

    /// Marks the input as finished, indicates that no more [`JxlDecoderSetInput`]
    /// will be called. This function allows the decoder to determine correctly if it
    /// should return success, need more input or error in certain cases. For
    /// backwards compatibility with a previous version of the API, using this
    /// function is optional when not using the [`JxlDecoderStatus::Box`] event (the decoder
    /// is able to determine the end of the image frames without marking the end),
    /// but using this function is required when using [`JxlDecoderStatus::Box`] for getting
    /// metadata box contents. This function does not replace [`JxlDecoderReleaseInput`],
    /// that function should still be called if its return value is needed.
    ///
    /// [`JxlDecoderCloseInput`] should be called as soon as all known input bytes
    /// are set (e.g. at the beginning when not streaming but setting all input
    /// at once), before the final [`JxlDecoderProcessInput`] calls.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    pub fn JxlDecoderCloseInput(dec: *mut JxlDecoder);

    /// Outputs the basic image information, such as image dimensions, bit depth and
    /// all other [`JxlBasicInfo`] fields, if available.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `info`: struct to copy the information into, or `NULL` to only check
    ///   whether the information is available through the return value.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the value is available
    /// - [`JxlDecoderStatus::NeedMoreInput`] if not yet available
    /// - [`JxlDecoderStatus::Error`] in case of other error conditions.
    pub fn JxlDecoderGetBasicInfo(
        dec: *const JxlDecoder,
        info: *mut JxlBasicInfo,
    ) -> JxlDecoderStatus;

    /// Outputs information for extra channel at the given index. The index must be
    /// smaller than `num_extra_channels` in the associated [`JxlBasicInfo`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `index`: index of the extra channel to query.
    /// - `info`: struct to copy the information into, or `NULL` to only check
    ///   whether the information is available through the return value.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the value is available
    /// - [`JxlDecoderStatus::NeedMoreInput`] if not yet available
    /// - [`JxlDecoderStatus::Error`] in case of other error conditions.
    pub fn JxlDecoderGetExtraChannelInfo(
        dec: *const JxlDecoder,
        index: usize,
        info: *mut JxlExtraChannelInfo,
    ) -> JxlDecoderStatus;

    /// Outputs name for extra channel at the given index in UTF-8. The index must be
    /// smaller than `num_extra_channels` in the associated [`JxlBasicInfo`]. The
    /// buffer for name must have at least `name_length + 1` bytes allocated, gotten
    /// from the associated [`JxlExtraChannelInfo`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `index`: index of the extra channel to query.
    /// - `name`: buffer to copy the name into
    /// - `size`: size of the name buffer in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the value is available
    /// - [`JxlDecoderStatus::NeedMoreInput`] if not yet available
    /// - [`JxlDecoderStatus::Error`] in case of other error conditions.
    pub fn JxlDecoderGetExtraChannelName(
        dec: *const JxlDecoder,
        index: usize,
        name: *mut c_char,
        size: usize,
    ) -> JxlDecoderStatus;

    /// Outputs the color profile as JPEG XL encoded structured data, if available.
    /// This is an alternative to an ICC Profile, which can represent a more limited
    /// amount of color spaces, but represents them exactly through enum values.
    ///
    /// It is often possible to use [`JxlDecoderGetColorAsICCProfile`] as an
    /// alternative anyway. The following scenarios are possible:
    /// - The JPEG XL image has an attached ICC Profile, in that case, the encoded
    ///   structured data is not available and this function will return an error
    ///   status. [`JxlDecoderGetColorAsICCProfile`] should be called instead.
    /// - The JPEG XL image has an encoded structured color profile, and it
    ///   represents an RGB or grayscale color space. This function will return it.
    ///   You can still use [`JxlDecoderGetColorAsICCProfile`] as well as an
    ///   alternative if desired, though depending on which RGB color space is
    ///   represented, the ICC profile may be a close approximation. It is also not
    ///   always feasible to deduce from an ICC profile which named color space it
    ///   exactly represents, if any, as it can represent any arbitrary space.
    ///   HDR color spaces such as those using PQ and HLG are also potentially
    ///   problematic, in that: while ICC profiles can encode a transfer function
    ///   that happens to approximate those of PQ and HLG (HLG for only one given
    ///   system gamma at a time, and necessitating a 3D LUT if gamma is to be
    ///   different from `1`), they cannot (before ICCv4.4) semantically signal that
    ///   this is the color space that they represent. Therefore, they will
    ///   typically not actually be interpreted as representing an HDR color space.
    ///   This is especially detrimental to PQ which will then be interpreted as if
    ///   the maximum signal value represented SDR white instead of 10000 cd/m^2,
    ///   meaning that the image will be displayed two orders of magnitude (5-7 EV)
    ///   too dim.
    /// - The JPEG XL image has an encoded structured color profile, and it
    ///   indicates an unknown or xyb color space. In that case, [`JxlDecoderGetColorAsICCProfile`]
    ///   is not available.
    ///
    /// When rendering an image on a system where ICC-based color management is used,
    /// [`JxlDecoderGetColorAsICCProfile`] should generally be used first as it will
    /// return a ready-to-use profile (with the aforementioned caveat about HDR).
    /// When knowledge about the nominal color space is desired if available, [`JxlDecoderGetColorAsEncodedProfile`]
    /// should be used first.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `target`: whether to get the original color profile from the metadata
    ///   or the color profile of the decoded pixels.
    /// - `color_encoding`: struct to copy the information into, or `NULL` to only
    ///   check whether the information is available through the return value.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the data is available and returned
    /// - [`JxlDecoderStatus::NeedMoreInput`] if not yet available
    /// - [`JxlDecoderStatus::Error`] in case the encoded structured color profile does not exist in the
    ///   codestream.
    pub fn JxlDecoderGetColorAsEncodedProfile(
        dec: *const JxlDecoder,
        target: JxlColorProfileTarget,
        color_encoding: *mut JxlColorEncoding,
    ) -> JxlDecoderStatus;

    /// Outputs the size in bytes of the ICC profile returned by [`JxlDecoderGetColorAsICCProfile`], if available,
    /// or indicates there is none available. In most cases, the image will have an ICC profile available, but
    /// if it does not, [`JxlDecoderGetColorAsEncodedProfile`] must be used instead.
    ///
    /// See [`JxlDecoderGetColorAsEncodedProfile`] for more information. The ICC
    /// profile is either the exact ICC profile attached to the codestream metadata,
    /// or a close approximation generated from JPEG XL encoded structured data,
    /// depending on what is encoded in the codestream.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `target`: whether to get the original color profile from the metadata
    ///   or the color profile of the decoded pixels.
    /// - `size`: variable to output the size into, or `NULL` to only check the
    ///   return status.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the ICC profile is available
    /// - [`JxlDecoderStatus::NeedMoreInput`] if the decoder has not yet received enough
    ///   input data to determine whether an ICC profile is available or what its
    ///   size is
    /// - [`JxlDecoderStatus::Error`] in case the ICC profile is not available and
    ///   cannot be generated.
    pub fn JxlDecoderGetICCProfileSize(
        dec: *const JxlDecoder,
        target: JxlColorProfileTarget,
        size: *mut usize,
    ) -> JxlDecoderStatus;

    /// Outputs ICC profile if available. The profile is only available if
    /// [`JxlDecoderGetICCProfileSize`] returns success. The output buffer must have
    /// at least as many bytes as given by [`JxlDecoderGetICCProfileSize`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `target`: whether to get the original color profile from the metadata
    ///   or the color profile of the decoded pixels.
    /// - `icc_profile`: buffer to copy the ICC profile into
    /// - `size`: size of the `icc_profile` buffer in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`]: if the profile was successfully returned
    /// - [`JxlDecoderStatus::NeedMoreInput`]: if not yet available
    /// - [`JxlDecoderStatus::Error`]: if the profile doesn't exist or the output size is not
    ///   large enough.
    pub fn JxlDecoderGetColorAsICCProfile(
        dec: *const JxlDecoder,
        target: JxlColorProfileTarget,
        icc_profile: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    /// Sets the desired output color profile of the decoded image by calling
    /// [`JxlDecoderSetOutputColorProfile`], passing on `color_encoding` and
    /// setting `icc_data` to `NULL`. See [`JxlDecoderSetOutputColorProfile`] for
    /// details.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `color_encoding`: the default color encoding to set
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the preference was set successfully, [`JxlDecoderStatus::Error`] otherwise.
    pub fn JxlDecoderSetPreferredColorProfile(
        dec: *mut JxlDecoder,
        color_encoding: *const JxlColorEncoding,
    ) -> JxlDecoderStatus;

    /// Requests that the decoder perform tone mapping to the peak display luminance
    /// passed as `desired_intensity_target`, if appropriate.
    ///
    /// # Note
    /// This is provided for convenience and the exact tone mapping that is
    /// performed is not meant to be considered authoritative in any way. It may
    /// change from version to version.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `desired_intensity_target`: the intended target peak luminance
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the preference was set successfully, [`JxlDecoderStatus::Error`] otherwise.
    pub fn JxlDecoderSetDesiredIntensityTarget(
        dec: *mut JxlDecoder,
        desired_intensity_target: f32,
    ) -> JxlDecoderStatus;

    /// Sets the desired output color profile of the decoded image either from a
    /// color encoding or an ICC profile. Valid calls of this function have either
    /// `color_encoding` or `icc_data` set to `NULL` and `icc_size` must be `0` if and
    /// only if `icc_data` is `NULL`.
    ///
    /// Depending on whether a color management system (CMS) has been set the
    /// behavior is as follows:
    ///
    /// If a color management system (CMS) has been set with [`JxlDecoderSetCms`],
    /// and the CMS supports output to the desired color encoding or ICC profile,
    /// then it will provide the output in that color encoding or ICC profile. If the
    /// desired color encoding or the ICC is not supported, then an error will be
    /// returned.
    ///
    /// If no CMS has been set with [`JxlDecoderSetCms`], there are two cases:
    ///
    /// 1. Calling this function with a color encoding will convert XYB images to
    ///    the desired color encoding. In this case, if the requested color encoding has
    ///    a narrower gamut, or the white points differ, then the resulting image can
    ///    have significant color distortion. Non-XYB images will not be converted to
    ///    the desired color space.
    ///
    /// 2. Calling this function with an ICC profile will result in an error.
    ///
    /// If called with an ICC profile (after a call to [`JxlDecoderSetCms`]), the
    /// ICC profile has to be a valid RGB or grayscale color profile.
    ///
    /// Can only be set after the [`JxlDecoderStatus::ColorEncoding`] event occurred and
    /// before any other event occurred, and should be used before getting
    /// [`JxlColorProfileTarget::Data`].
    ///
    /// This function must not be called before [`JxlDecoderSetCms`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `color_encoding`: the output color encoding
    /// - `icc_data`: bytes of the icc profile
    /// - `icc_size`: size of the icc profile in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the color profile was set successfully,
    ///   [`JxlDecoderStatus::Error`] otherwise.
    pub fn JxlDecoderSetOutputColorProfile(
        dec: *mut JxlDecoder,
        color_encoding: *const JxlColorEncoding,
        icc_data: *const u8,
        icc_size: usize,
    ) -> JxlDecoderStatus;

    /// Sets the color management system (CMS) that will be used for color
    /// conversion (if applicable) during decoding. May only be set before starting
    /// decoding and must not be called after [`JxlDecoderSetOutputColorProfile`].
    ///
    /// See [`JxlDecoderSetOutputColorProfile`] for how color conversions are done
    /// depending on whether or not a CMS has been set with [`JxlDecoderSetCms`].
    ///
    /// # Parameters
    /// - `dec`: decoder object.
    /// - `cms`: structure representing a CMS implementation. See [`JxlCmsInterface`] for more details.
    pub fn JxlDecoderSetCms(dec: *mut JxlDecoder, cms: JxlCmsInterface) -> JxlDecoderStatus;

    /// Returns the minimum size in bytes of the preview image output pixel buffer
    /// for the given format. This is the buffer for [`JxlDecoderSetPreviewOutBuffer`].
    /// Requires the preview header information is available in the decoder.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `format`: format of pixels
    /// - `size`: output value, buffer size in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success
    /// - [`JxlDecoderStatus::Error`] on error, such as information not available yet.
    pub fn JxlDecoderPreviewOutBufferSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        size: *mut usize,
    ) -> JxlDecoderStatus;

    /// Sets the buffer to write the low-resolution preview image
    /// to. The size of the buffer must be at least as large as given by [`JxlDecoderPreviewOutBufferSize`].
    /// The buffer follows the format described by [`JxlPixelFormat`]. The preview image dimensions are given by the
    /// [`JxlPreviewHeader`]. The buffer is owned by the caller.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `format`: format of pixels. Object owned by user and its contents are
    ///   copied internally.
    /// - `buffer`: buffer type to output the pixel data to
    /// - `size`: size of buffer in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success, [`JxlDecoderStatus::Error`] on error, such as
    ///   size too small.
    pub fn JxlDecoderSetPreviewOutBuffer(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
    ) -> JxlDecoderStatus;

    /// Outputs the information from the frame, such as duration when `have_animation`.
    /// This function can be called when [`JxlDecoderStatus::Frame`] occurred for the current
    /// frame, even when `have_animation` in the [`JxlBasicInfo`] is [`JxlBool::False`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `header`: struct to copy the information into, or `NULL` to only check
    ///   whether the information is available through the return value.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the value is available
    /// - [`JxlDecoderStatus::NeedMoreInput`] if not yet available
    /// - [`JxlDecoderStatus::Error`] in case of other error conditions.
    pub fn JxlDecoderGetFrameHeader(
        dec: *const JxlDecoder,
        header: *mut JxlFrameHeader,
    ) -> JxlDecoderStatus;

    /// Outputs name for the current frame. The buffer for name must have at least
    /// `name_length + 1` bytes allocated, gotten from the associated [`JxlFrameHeader`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `name`: buffer to copy the name into
    /// - `size`: size of the name buffer in bytes, including zero termination
    ///   character, so this must be at least [`JxlFrameHeader::name_length`] + 1.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the value is available
    /// - [`JxlDecoderStatus::NeedMoreInput`] if not yet available
    /// - [`JxlDecoderStatus::Error`] in case of other error conditions.
    pub fn JxlDecoderGetFrameName(
        dec: *const JxlDecoder,
        name: *mut c_char,
        size: usize,
    ) -> JxlDecoderStatus;

    /// Outputs the blend information for the current frame for a specific extra
    /// channel. This function can be called once the [`JxlDecoderStatus::Frame`] event occurred
    /// for the current frame, even if the `have_animation` field in the [`JxlBasicInfo`] is [`JxlBool::False`].
    /// This information is only useful if coalescing is disabled; otherwise the decoder will have performed
    /// blending already.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `index`: the index of the extra channel
    /// - `blend_info`: struct to copy the information into
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success
    /// - [`JxlDecoderStatus::Error`] on error
    pub fn JxlDecoderGetExtraChannelBlendInfo(
        dec: *const JxlDecoder,
        index: usize,
        blend_info: *mut JxlBlendInfo,
    );

    /// Returns the minimum size in bytes of the image output pixel buffer for the
    /// given format. This is the buffer for [`JxlDecoderSetImageOutBuffer`].
    /// Requires that the basic image information is available in the decoder in the
    /// case of coalescing enabled (default). In case coalescing is disabled, this
    /// can only be called after the [`JxlDecoderStatus::Frame`] event occurs. In that case,
    /// it will return the size required to store the possibly cropped frame (which
    /// can be larger or smaller than the image dimensions).
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `format`: format of the pixels.
    /// - `size`: output value, buffer size in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success, [`JxlDecoderStatus::Error`] on error, such as
    ///   information not available yet.
    pub fn JxlDecoderImageOutBufferSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        size: *mut usize,
    ) -> JxlDecoderStatus;

    /// Sets the buffer to write the full resolution image to. This can be set when
    /// the [`JxlDecoderStatus::Frame`] event occurs, must be set when the
    /// [`JxlDecoderStatus::NeedImageOutBuffer`] event occurs, and applies only for the
    /// current frame. The size of the buffer must be at least as large as given
    /// by [`JxlDecoderImageOutBufferSize`]. The buffer follows the format described
    /// by [`JxlPixelFormat`]. The buffer is owned by the caller.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `format`: format of the pixels. Object owned by user and its contents
    ///   are copied internally.
    /// - `buffer`: buffer type to output the pixel data to
    /// - `size`: size of buffer in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success, [`JxlDecoderStatus::Error`] on error, such as
    ///   size too small.
    pub fn JxlDecoderSetImageOutBuffer(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
    ) -> JxlDecoderStatus;

    /// Sets pixel output callback. This is an alternative to [`JxlDecoderSetImageOutBuffer`].
    /// This can be set when the [`JxlDecoderStatus::Frame`] event occurs, must be set when the
    /// [`JxlDecoderStatus::NeedImageOutBuffer`] event occurs, and applies only for the current frame.
    /// Only one of [`JxlDecoderSetImageOutBuffer`] or [`JxlDecoderSetImageOutCallback`] may be used
    /// for the same frame, not both at the same time.
    ///
    /// The callback will be called multiple times, to receive the image
    /// data in small chunks. The callback receives a horizontal stripe of pixel
    /// data, `1` pixel high, xsize pixels wide, called a scanline. The xsize here is
    /// not the same as the full image width, the scanline may be a partial section,
    /// and xsize may differ between calls. The user can then process and/or copy the
    /// partial scanline to an image buffer. The callback may be called
    /// simultaneously by different threads when using a threaded parallel runner, on
    /// different pixels.
    ///
    /// If [`JxlDecoderFlushImage`] is not used, then each pixel will be visited
    /// exactly once by the different callback calls, during processing with one or
    /// more [`JxlDecoderProcessInput`] calls. These pixels are decoded to full
    /// detail, they are not part of a lower resolution or lower quality progressive
    /// pass, but the final pass.
    ///
    /// If [`JxlDecoderFlushImage`] is used, then in addition each pixel will be
    /// visited zero or one times during the blocking [`JxlDecoderFlushImage`] call.
    /// Pixels visited as a result of [`JxlDecoderFlushImage`] may represent a lower
    /// resolution or lower quality intermediate progressive pass of the image. Any
    /// visited pixel will be of a quality at least as good or better than previous
    /// visits of this pixel. A pixel may be visited zero times if it cannot be
    /// decoded yet or if it was already decoded to full precision (this behavior is
    /// not guaranteed).
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `format`: format of the pixels. Object owned by user; its contents are
    ///   copied internally.
    /// - `callback`: the callback function receiving partial scanlines of pixel
    ///   data.
    /// - `opaque`: optional user data, which will be passed on to the callback,
    ///   may be `NULL`.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success, [`JxlDecoderStatus::Error`] on error, such
    ///   as [`JxlDecoderSetImageOutBuffer`] already set.
    pub fn JxlDecoderSetImageOutCallback(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        callback: JxlImageOutCallback,
        opaque: *mut c_void,
    ) -> JxlDecoderStatus;

    /// Similar to [`JxlDecoderSetImageOutCallback`] except that the callback is
    /// allowed an initialization phase during which it is informed of how many
    /// threads will call it concurrently, and those calls are further informed of
    /// which thread they are occurring in.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `format`: format of the pixels. Object owned by user; its contents are
    ///   copied internally.
    /// - `init_callback`: initialization callback.
    /// - `run_callback`: the callback function receiving partial scanlines of
    ///   pixel data.
    /// - `destroy_callback`: clean-up callback invoked after all calls to `run_callback`.
    ///   May be `NULL` if no clean-up is necessary.
    /// - `init_opaque`: optional user data passed to `init_callback`, may be `NULL`
    ///   (unlike the return value from `init_callback` which may only be `NULL` if
    ///   initialization failed).
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success, [`JxlDecoderStatus::Error`] on error, such
    ///   as [`JxlDecoderSetImageOutBuffer`] having already been called.
    pub fn JxlDecoderSetMultithreadedImageOutCallback(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        init_callback: JxlImageOutInitCallback,
        run_callback: JxlImageOutRunCallback,
        destroy_callback: JxlImageOutDestroyCallback,
        init_opaque: *mut c_void,
    ) -> JxlDecoderStatus;

    /// Returns the minimum size in bytes of an extra channel pixel buffer for the
    /// given format. This is the buffer for [`JxlDecoderSetExtraChannelBuffer`].
    /// Requires the basic image information is available in the decoder.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `format`: format of the pixels. The `num_channels` value is ignored and is
    ///   always treated to be `1`.
    /// - `size`: output value, buffer size in bytes
    /// - `index`: which extra channel to get, matching the index used in [`JxlDecoderGetExtraChannelInfo`].
    ///   Must be smaller than `num_extra_channels` in the associated [`JxlBasicInfo`].
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success
    /// - [`JxlDecoderStatus::Error`] on error, such as information not available yet or invalid index.
    pub fn JxlDecoderExtraChannelBufferSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        size: *mut usize,
        index: u32,
    ) -> JxlDecoderStatus;

    /// Sets the buffer to write an extra channel to. This can be set when
    /// the [`JxlDecoderStatus::Frame`] or [`JxlDecoderStatus::NeedImageOutBuffer`] event occurs,
    /// and applies only for the current frame. The size of the buffer must be at
    /// least as large as given by [`JxlDecoderExtraChannelBufferSize`]. The buffer
    /// follows the format described by [`JxlPixelFormat`], but where `num_channels`
    /// is `1`. The buffer is owned by the caller. The amount of extra channels is
    /// given by the `num_extra_channels` field in the associated [`JxlBasicInfo`],
    /// and the information of individual extra channels can be queried with [`JxlDecoderGetExtraChannelInfo`].
    /// To get multiple extra channels, this function must be called multiple times, once for each wanted index.
    /// Not all images have extra channels. The alpha channel is an extra channel and can be gotten
    /// as part of the color channels when using an RGBA pixel buffer with [`JxlDecoderSetImageOutBuffer`],
    /// but additionally also can be gotten separately as extra channel. The color channels themselves cannot be gotten
    /// this way.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `format`: format of the pixels. Object owned by user and its contents
    ///   are copied internally. The `num_channels` value is ignored and is always
    ///   treated to be `1`.
    /// - `buffer`: buffer type to output the pixel data to
    /// - `size`: size of buffer in bytes
    /// - `index`: which extra channel to get, matching the index used in [`JxlDecoderGetExtraChannelInfo`].
    ///   Must be smaller than `num_extra_channels` in the associated [`JxlBasicInfo`].
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success, [`JxlDecoderStatus::Error`] on error, such as
    ///   size too small or invalid index.
    pub fn JxlDecoderSetExtraChannelBuffer(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
        index: u32,
    ) -> JxlDecoderStatus;

    /// Sets output buffer for reconstructed JPEG codestream.
    ///
    /// The data is owned by the caller and may be used by the decoder until [`JxlDecoderReleaseJPEGBuffer`]
    /// is called or the decoder is destroyed or reset so must be kept alive until then.
    ///
    /// If a JPEG buffer was set before and released with [`JxlDecoderReleaseJPEGBuffer`],
    /// bytes that the decoder has already output should not be included,
    /// only the remaining bytes output must be set.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `data`: pointer to next bytes to write to
    /// - `size`: amount of bytes available starting from data
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Error`] if output buffer was already set and [`JxlDecoderReleaseJPEGBuffer`]
    ///   was not called on it, [`JxlDecoderStatus::Success`] otherwise
    pub fn JxlDecoderSetJPEGBuffer(
        dec: *mut JxlDecoder,
        data: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    /// Releases buffer which was provided with [`JxlDecoderSetJPEGBuffer`].
    ///
    /// Calling [`JxlDecoderReleaseJPEGBuffer`] is required whenever
    /// a buffer is already set and a new buffer needs to be added with [`JxlDecoderSetJPEGBuffer`],
    /// but is not required before [`JxlDecoderDestroy`] or [`JxlDecoderReset`].
    ///
    /// Calling [`JxlDecoderReleaseJPEGBuffer`] when no buffer is set is
    /// not an error and returns `0`.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    ///
    /// # Returns
    /// - The amount of bytes the decoder has not yet written to of the data
    ///   set by [`JxlDecoderSetJPEGBuffer`], or `0` if no buffer is set or
    ///   [`JxlDecoderReleaseJPEGBuffer`] was already called.
    pub fn JxlDecoderReleaseJPEGBuffer(dec: *mut JxlDecoder) -> usize;

    /// Sets output buffer for box output codestream.
    ///
    /// The data is owned by the caller and may be used by the decoder until [`JxlDecoderReleaseBoxBuffer`]
    /// is called or the decoder is destroyed or reset so must be kept alive until then.
    ///
    /// If for the current box a box buffer was set before and released with [`JxlDecoderReleaseBoxBuffer`],
    /// bytes that the decoder has already output should not be included, only the remaining bytes output must be set.
    ///
    /// The [`JxlDecoderReleaseBoxBuffer`] must be used at the next [`JxlDecoderStatus::Box`] event or final
    /// [`JxlDecoderStatus::Success`] event to compute the size of the output box bytes.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `data`: pointer to next bytes to write to
    /// - `size`: amount of bytes available starting from data
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Error`] if output buffer was already set and [`JxlDecoderReleaseBoxBuffer`]
    ///   was not called on it, [`JxlDecoderStatus::Success`] otherwise
    pub fn JxlDecoderSetBoxBuffer(
        dec: *mut JxlDecoder,
        data: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    /// Releases buffer which was provided with [`JxlDecoderSetBoxBuffer`].
    ///
    /// Calling [`JxlDecoderReleaseBoxBuffer`] is required whenever a buffer is already set and
    /// a new buffer needs to be added with [`JxlDecoderSetBoxBuffer`], but is not required before
    /// [`JxlDecoderDestroy`] or [`JxlDecoderReset`].
    ///
    /// Calling [`JxlDecoderReleaseBoxBuffer`] when no buffer is set is not an error and returns `0`.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    ///
    /// # Returns
    /// - The amount of bytes the decoder has not yet written to of the data set by [`JxlDecoderSetBoxBuffer`],
    ///   or `0` if no buffer is set or [`JxlDecoderReleaseBoxBuffer`] was already called.,
    pub fn JxlDecoderReleaseBoxBuffer(dec: *mut JxlDecoder) -> usize;

    /// Configures whether to get boxes in raw mode or in decompressed mode. In raw
    /// mode, boxes are output as their bytes appear in the container file, which may
    /// be decompressed, or compressed if their type is "brob". In decompressed mode,
    /// "brob" boxes are decompressed with Brotli before outputting them. The size of
    /// the decompressed stream is not known before the decompression has already
    /// finished.
    ///
    /// The default mode is raw. This setting can only be changed before decoding, or
    /// directly after a [`JxlDecoderStatus::Box`] event, and is remembered until the decoder
    /// is reset or destroyed.
    ///
    /// Enabling decompressed mode requires Brotli support from the library.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `decompress`: [`JxlBool::True`] to transparently decompress, [`JxlBool::False`]
    ///   to get boxes in raw mode.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Error`] if decompressed mode is set and Brotli is not
    ///   available, [`JxlDecoderStatus::Success`] otherwise.
    pub fn JxlDecoderSetDecompressBoxes(
        dec: *mut JxlDecoder,
        decompress: JxlBool,
    ) -> JxlDecoderStatus;

    /// Outputs the type of the current box, after a [`JxlDecoderStatus::Box`] event occurred,
    /// as `4` characters without null termination character. In case of a compressed
    /// "brob" box, this will return "brob" if the decompressed argument is
    /// [`JxlBool::False`], or the underlying box type if the decompressed argument is
    /// [`JxlBool::True`].
    ///
    /// The following box types are currently described in ISO/IEC 18181-2:
    ///  - "Exif": a box with EXIF metadata.  Starts with a 4-byte tiff header offset
    ///    (big-endian `uint32`) that indicates the start of the actual EXIF data
    ///    (which starts with a tiff header). Usually the offset will be zero and the
    ///    EXIF data starts immediately after the offset field. The Exif orientation
    ///    should be ignored by applications; the JPEG XL codestream orientation
    ///    takes precedence and libjxl will by default apply the correct orientation
    ///    automatically (see [`JxlDecoderSetKeepOrientation`]).
    ///  - "xml ": a box with XML data, in particular XMP metadata.
    ///  - "jumb": a JUMBF superbox (JPEG Universal Metadata Box Format, ISO/IEC
    ///    19566-5).
    ///  - "JXL ": mandatory signature box, must come first, `12` bytes long
    ///    including the box header
    ///  - "ftyp": a second mandatory signature box, must come second, `20` bytes
    ///    long including the box header
    ///  - "jxll": a JXL level box. This indicates if the codestream is level `5` or
    ///    level `10` compatible. If not present, it is level `5`. Level `10` allows
    ///    more features such as very high image resolution and bit-depths above `16`
    ///    bits per channel. Added automatically by the encoder when
    ///    [`crate::encoder::encode::JxlEncoderSetCodestreamLevel`] is used
    ///  - "jxlc": a box with the image codestream, in case the codestream is not
    ///    split across multiple boxes. The codestream contains the JPEG XL image
    ///    itself, including the basic info such as image dimensions, ICC color
    ///    profile, and all the pixel data of all the image frames.
    ///  - "jxlp": a codestream box in case it is split across multiple boxes.
    ///    The contents are the same as in case of a jxlc box, when concatenated.
    ///  - "brob": a Brotli-compressed box, which otherwise represents an existing
    ///    type of box such as Exif or "xml ". When [`JxlDecoderSetDecompressBoxes`]
    ///    is set to [`JxlBool::True`], these boxes will be transparently decompressed by the
    ///    decoder.
    ///  - "jxli": frame index box, can list the keyframes in case of a JPEG XL
    ///    animation allowing the decoder to jump to individual frames more
    ///    efficiently.
    ///  - "jbrd": JPEG reconstruction box, contains the information required to
    ///    byte-for-byte losslessly reconstruct a JPEG-1 image. The JPEG DCT
    ///    coefficients (pixel content) themselves as well as the ICC profile are
    ///    encoded in the JXL codestream (jxlc or jxlp) itself. EXIF, XMP and JUMBF
    ///    metadata is encoded in the corresponding boxes. The jbrd box itself
    ///    contains information such as the remaining app markers of the JPEG-1 file
    ///    and everything else required to fit the information together into the
    ///    exact original JPEG file.
    ///
    /// Other application-specific boxes can exist. Their typename should not begin
    /// with "jxl" or "JXL" or conflict with other existing typenames.
    ///
    /// The signature, jxl* and jbrd boxes are processed by the decoder and would
    /// typically be ignored by applications. The typical way to use this function is
    /// to check if an encountered box contains metadata that the application is
    /// interested in (e.g. EXIF or XMP metadata), in order to conditionally set a
    /// box buffer.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `type`: buffer to copy the type into
    /// - `decompressed`: which box type to get: [`JxlBool::False`] to get the raw box type,
    ///   which can be `"brob"`, [`JxlBool::True`] to get the underlying box type.
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if the value is available
    /// - [`JxlDecoderStatus::Error`] if not, for example the JPEG XL file does not use the container format.
    pub fn JxlDecoderGetBoxType(
        dec: *mut JxlDecoder,
        box_type: &mut JxlBoxType,
        decompressed: JxlBool,
    ) -> JxlDecoderStatus;

    /// Returns the size of a box as it appears in the container file, after the
    /// [`JxlDecoderStatus::Box`] event. This includes all the box headers.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `size`: raw size of the box in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Error`] if no box size is available, [`JxlDecoderStatus::Success`] otherwise.
    pub fn JxlDecoderGetBoxSizeRaw(dec: *mut JxlDecoder, size: *mut u64) -> JxlDecoderStatus;

    /// Returns the size of the contents of a box, after the [`JxlDecoderStatus::Box`] event.
    /// This does not include any of the headers of the box. For compressed "brob" boxes,
    /// this is the size of the compressed content. Even when [`JxlDecoderSetDecompressBoxes`] is enabled,
    /// the return value of function does not change, and the decompressed size is not known before it
    /// has already been decompressed and output.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `size`: size of the payload of the box in bytes
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Error`] if no box size is available, [`JxlDecoderStatus::Success`]
    ///   otherwise.
    pub fn JxlDecoderGetBoxSizeContents(dec: *mut JxlDecoder, size: *mut u64) -> JxlDecoderStatus;

    /// Configures at which progressive steps in frame decoding the [`JxlDecoderStatus::FrameProgression`] event occurs.
    /// The default value for the level of detail if this function is never called is [`JxlProgressiveDetail::DC`].
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `detail`: at which level of detail to trigger [`JxlDecoderStatus::FrameProgression`]
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success, [`JxlDecoderStatus::Error`] on error, such as
    ///   an invalid value for the progressive detail.
    pub fn JxlDecoderSetProgressiveDetail(
        dec: *mut JxlDecoder,
        detail: JxlProgressiveDetail,
    ) -> JxlDecoderStatus;

    /// Returns the intended downsampling ratio for the progressive frame produced
    /// by [`JxlDecoderFlushImage`] after the latest [`JxlDecoderStatus::FrameProgression`] event.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    ///
    /// # Returns
    /// The intended downsampling ratio, can be `1`, `2`, `4` or `8`.
    pub fn JxlDecoderGetIntendedDownsamplingRatio(dec: *const JxlDecoder) -> usize;

    /// Outputs progressive step towards the decoded image so far when only partial
    /// input was received. If the flush was successful, the buffer set with [`JxlDecoderSetImageOutBuffer`]
    /// will contain partial image data.
    ///
    /// Can be called when [`JxlDecoderProcessInput`] returns [`JxlDecoderStatus::NeedMoreInput`], after the
    /// [`JxlDecoderStatus::Frame`] event already occurred and before the [`JxlDecoderStatus::FullImage`] event
    /// occurred for a frame.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] if image data was flushed to the output buffer,
    /// - [`JxlDecoderStatus::Error`] when no flush was done, e.g., if not enough image data was available yet
    ///   even for flush, or no output buffer was set yet. This error is not fatal, it only indicates no flushed
    ///   image is available right now. Regular decoding can still be performed.
    pub fn JxlDecoderFlushImage(dec: *mut JxlDecoder) -> JxlDecoderStatus;

    /// Sets the bit depth of the output buffer or callback.
    ///
    /// Can be called after [`JxlDecoderSetImageOutBuffer`] or
    /// [`JxlDecoderSetImageOutCallback`]. For float pixel data types, only the default
    /// [`JxlBitDepthType::FromPixelFormat`] setting is supported.
    ///
    /// # Parameters
    /// - `dec`: decoder object
    /// - `bit_depth`: the bit depth setting of the pixel output
    ///
    /// # Returns
    /// - [`JxlDecoderStatus::Success`] on success, [`JxlDecoderStatus::Error`] on error, such as
    ///   incompatible custom bit depth and pixel data type.
    pub fn JxlDecoderSetImageOutBitDepth(
        dec: *mut JxlDecoder,
        bit_depth: *const JxlBitDepth,
    ) -> JxlDecoderStatus;
}
