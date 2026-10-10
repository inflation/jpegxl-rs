/*
 * This file is part of jpegxl-rs.
 *
 * jpegxl-rs is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * jpegxl-rs is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
 */

use jpegxl_sys::{
    color::color_encoding::JxlColorEncoding,
    common::types::{JxlBoxType, JxlDataType, JxlPixelFormat},
    decode as d,
    metadata::codestream_header::{JxlExtraChannelInfo, JxlFrameHeader},
};
use std::mem::MaybeUninit;

use super::{
    BasicInfo, BoxHeader, ColorProfileTarget, Event, Events, FrameInfo, Image, JxlDecoder,
    PixelFormat,
};
use eros::{type_set::SupersetOf, IntoUnion, ReshapeUnion, TypeSet};

use super::{
    check_dec_status, GenericError, IncompleteInput, NotAvailableYet, UnsupportedBitWidth,
};
use crate::errors::{InternalError, InvalidState};

/// Minimum number of bytes of a new chunk glued to the bytes libjxl left unprocessed.
/// The glued part grows with the carry, so a large section is not copied over and over.
const CARRY_PREFIX: usize = 64;

fn data_type_of(bits: u32, exponent_bits: u32) -> Result<JxlDataType, UnsupportedBitWidth> {
    match (bits, exponent_bits) {
        (x, 0) if x <= 8 => Ok(JxlDataType::Uint8),
        (x, 0) if x <= 16 => Ok(JxlDataType::Uint16),
        (16, _) => Ok(JxlDataType::Float16),
        (32, _) => Ok(JxlDataType::Float),
        (x, _) => Err(UnsupportedBitWidth(x)),
    }
}

/// Output buffer owned by the session while libjxl holds a pointer to it
struct Slot {
    format: JxlPixelFormat,
    data: Vec<u8>,
}

impl Slot {
    fn into_image(self) -> Image {
        Image {
            format: self.format,
            data: self.data,
        }
    }
}

/// Zero-filled buffer that grows when libjxl runs out of room
struct Sink {
    buf: Vec<u8>,
    written: usize,
}

impl Sink {
    fn new(size: usize) -> Self {
        Self {
            buf: vec![0; size.max(1)],
            written: 0,
        }
    }

    fn attach<S, I>(
        &mut self,
        set: impl FnOnce(*mut u8, usize) -> d::JxlDecoderStatus,
    ) -> eros::Result<(), S>
    where
        S: TypeSet,
        S::Variants: SupersetOf<<(GenericError,) as TypeSet>::Variants, I>,
    {
        let rest = &mut self.buf[self.written..];
        check_dec_status::<(GenericError,), _, _>(set(rest.as_mut_ptr(), rest.len())).widen()
    }

    /// `remaining` is what libjxl returned from releasing the buffer
    fn grow(&mut self, remaining: usize) {
        self.written = self.buf.len() - remaining;
        self.buf.resize((self.buf.len() * 2).max(64), 0);
    }

    fn finish(mut self, remaining: usize) -> Vec<u8> {
        self.buf.truncate(self.buf.len() - remaining);
        self.buf
    }
}

/// A decoding session that is driven by the caller.
///
/// [`process`](Self::process) runs the decoder until the next [`Event`].
/// The caller reacts to it, e.g. by providing an output buffer, and calls `process` again
/// until [`Event::Success`].
///
/// ```
/// # use jpegxl_rs::{decoder_builder, decode::{Event, Events}};
/// # fn main() -> jpegxl_rs::eros::Result<()> {
/// # let data = include_bytes!("../../../samples/sample.jxl");
/// let mut decoder = decoder_builder().build()?;
/// let mut session = decoder.session(Events::FULL_IMAGE)?;
/// let mut input = &data[..];
/// let mut frames = vec![];
/// loop {
///     match session.process(&mut input)? {
///         Event::NeedImageOutBuffer => session.alloc_image_buffer(Default::default(), None)?,
///         Event::FullImage(Some(image)) => frames.push(image.into_pixels()),
///         Event::Success => break,
///         _ => {}
///     }
/// }
/// assert_eq!(frames.len(), 1);
/// # Ok(()) }
/// ```
///
/// Dropping the session resets the decoder, so it can be reused.
pub struct Session<'dec, 'pr, 'mm> {
    dec: &'dec JxlDecoder<'pr, 'mm>,
    events: Events,
    basic_info: Option<BasicInfo>,
    /// Unprocessed bytes libjxl asked to be provided again
    carry: Vec<u8>,
    /// Input owned by the session once it is closed
    tail: Option<Vec<u8>>,
    closing: bool,
    image: Option<Slot>,
    /// Extra channel buffers of the current frame, with their channel index
    extra: Vec<(u32, Slot)>,
    /// The frame is decoded, so `extra` holds final pixels that libjxl no longer writes to
    extra_ready: bool,
    preview: Option<Slot>,
    jpeg: Option<Sink>,
    boxed: Option<Sink>,
}

impl<'dec, 'pr, 'mm> Session<'dec, 'pr, 'mm> {
    pub(crate) fn new(
        dec: &'dec JxlDecoder<'pr, 'mm>,
        events: Events,
    ) -> eros::Result<Self, (GenericError,)> {
        let mut events = events | Events::BASIC_INFO;
        if events.contains(Events::BOX) {
            // The collected box is only handed back by `BoxComplete`
            events |= Events::BOX_COMPLETE;
        }
        dec.setup_decoder(events)?;

        Ok(Self {
            dec,
            events,
            basic_info: None,
            carry: Vec::new(),
            tail: None,
            closing: false,
            image: None,
            extra: Vec::new(),
            extra_ready: false,
            preview: None,
            jpeg: None,
            boxed: None,
        })
    }

    /// Basic information, available after [`Event::BasicInfo`]
    #[must_use]
    pub fn basic_info(&self) -> Option<&BasicInfo> {
        self.basic_info.as_ref()
    }

    /// Run the decoder until the next [`Event`].
    ///
    /// `input` is advanced past the bytes the decoder consumed, so on any event except
    /// [`Event::NeedMoreInput`] it may still hold bytes that the next call will process.
    /// On `NeedMoreInput` it is always empty.
    ///
    /// # Errors
    /// Return a [`GenericError`] if the decoder fails, or [`IncompleteInput`] if the input
    /// is closed and incomplete.
    pub fn process(
        &mut self,
        input: &mut &[u8],
    ) -> eros::Result<Event, (GenericError, IncompleteInput, InternalError)> {
        loop {
            let status = self.run(input).widen()?;
            if let Some(event) = self.handle(status)? {
                return Ok(event);
            }
        }
    }

    /// Declare that no input follows what is passed to the next
    /// [`process`](Self::process) call. Required to get every [`Event::Box`].
    ///
    /// The bytes passed to that call are copied once, because libjxl keeps using them
    /// after the input is closed.
    pub fn close_input(&mut self) {
        self.closing = true;
    }

    /// Calls libjxl, taking care of feeding the input
    fn run(&mut self, input: &mut &[u8]) -> eros::Result<d::JxlDecoderStatus, (GenericError,)> {
        use d::JxlDecoderStatus as s;
        let dec = self.dec.dec;

        if self.tail.is_some() {
            // SAFETY: the decoder is valid while the session borrows it
            return Ok(unsafe { d::JxlDecoderProcessInput(dec) });
        }

        if self.closing {
            let mut rest = std::mem::take(&mut self.carry);
            rest.extend_from_slice(input);
            *input = &[];
            // SAFETY: `self.tail` keeps `rest` until the reset on drop
            check_dec_status(unsafe { d::JxlDecoderSetInput(dec, rest.as_ptr(), rest.len()) })?;
            // SAFETY: the decoder is valid while the session borrows it
            unsafe { d::JxlDecoderCloseInput(dec) };
            self.tail = Some(rest);
            // SAFETY: the decoder is valid while the session borrows it
            return Ok(unsafe { d::JxlDecoderProcessInput(dec) });
        }

        loop {
            let carried = self.carry.len();
            let prefix = if carried == 0 {
                input.len()
            } else {
                input.len().min(carried.max(CARRY_PREFIX))
            };
            if carried > 0 {
                self.carry.extend_from_slice(&input[..prefix]);
            }
            let view: &[u8] = if carried == 0 { input } else { &self.carry };

            // SAFETY: `view` outlives the `ReleaseInput` below
            check_dec_status(unsafe { d::JxlDecoderSetInput(dec, view.as_ptr(), view.len()) })?;
            // SAFETY: the decoder is valid while the session borrows it
            let status = unsafe { d::JxlDecoderProcessInput(dec) };
            // SAFETY: the decoder is valid while the session borrows it
            let consumed = view.len() - unsafe { d::JxlDecoderReleaseInput(dec) };

            if status == s::NeedMoreInput {
                if carried == 0 {
                    self.carry = input[consumed..].to_vec();
                } else {
                    self.carry.drain(..consumed);
                }
                *input = &input[prefix..];
                if input.is_empty() {
                    return Ok(status);
                }
            } else {
                self.carry.truncate(carried);
                if consumed >= carried {
                    self.carry.clear();
                    *input = &input[consumed - carried..];
                } else {
                    self.carry.drain(..consumed);
                }
                return Ok(status);
            }
        }
    }

    fn handle(
        &mut self,
        status: d::JxlDecoderStatus,
    ) -> eros::Result<Option<Event>, (GenericError, IncompleteInput, InternalError)> {
        use d::JxlDecoderStatus as s;
        let dec = self.dec.dec;

        Ok(Some(match status {
            s::Error => return Err(GenericError).union(),
            s::NeedMoreInput if self.tail.is_some() => return Err(IncompleteInput).union(),
            s::NeedMoreInput => Event::NeedMoreInput,
            s::Success => {
                self.release_box();
                Event::Success
            }
            s::BasicInfo => {
                let mut info = MaybeUninit::uninit();
                // SAFETY: the decoder is valid while the session borrows it
                let status = unsafe { d::JxlDecoderGetBasicInfo(dec, info.as_mut_ptr()) };
                // Its only failure is `NeedMoreInput`, which cannot happen on this event
                if status != s::Success {
                    return Err(InternalError("basic info is missing on Event::BasicInfo")).union();
                }
                // SAFETY: `GetBasicInfo` succeeded
                let info = unsafe { info.assume_init() };

                if let Some(pr) = self.dec.parallel_runner {
                    pr.callback_basic_info(&info);
                }
                self.basic_info = Some(info.clone());
                Event::BasicInfo(info)
            }
            s::ColorEncoding => Event::ColorEncoding,
            s::NeedPreviewOutBuffer => Event::NeedPreviewOutBuffer,
            // libjxl is done with the preview buffer once this event is emitted
            s::PreviewImage => Event::PreviewImage(self.preview.take().map(Slot::into_image)),
            s::Frame => Event::Frame(self.frame_info().widen()?),
            s::NeedImageOutBuffer => Event::NeedImageOutBuffer,
            s::FrameProgression => Event::FrameProgression,
            s::FullImage => {
                self.extra_ready = true;
                match self.jpeg.take() {
                    Some(sink) => {
                        // SAFETY: the decoder is valid while the session borrows it
                        let remaining = unsafe { d::JxlDecoderReleaseJPEGBuffer(dec) };
                        Event::Jpeg(sink.finish(remaining))
                    }
                    // libjxl is done with the frame buffer once this event is emitted; it asks
                    // for a new one with `NeedImageOutBuffer` before writing the next frame
                    None => Event::FullImage(self.image.take().map(Slot::into_image)),
                }
            }
            s::JPEGReconstruction => {
                let mut sink = Sink::new(self.dec.init_jpeg_buffer);
                // SAFETY: `self.jpeg` keeps the buffer until it is released
                sink.attach(|ptr, len| unsafe { d::JxlDecoderSetJPEGBuffer(dec, ptr, len) })?;
                self.jpeg = Some(sink);
                Event::JpegReconstruction
            }
            s::JPEGNeedMoreOutput => {
                let sink = self
                    .jpeg
                    .as_mut()
                    .ok_or(InternalError("JPEG output requested without a JPEG buffer"))
                    .union()?;
                // SAFETY: the decoder is valid while the session borrows it
                sink.grow(unsafe { d::JxlDecoderReleaseJPEGBuffer(dec) });
                // SAFETY: `self.jpeg` keeps the buffer until it is released
                sink.attach(|ptr, len| unsafe { d::JxlDecoderSetJPEGBuffer(dec, ptr, len) })?;
                return Ok(None);
            }
            s::Box => {
                self.release_box();
                Event::Box(self.box_header().widen()?)
            }
            s::BoxNeedMoreOutput => {
                let sink = self
                    .boxed
                    .as_mut()
                    .ok_or(InternalError("Box output requested without a box buffer"))
                    .union()?;
                // SAFETY: the decoder is valid while the session borrows it
                sink.grow(unsafe { d::JxlDecoderReleaseBoxBuffer(dec) });
                // SAFETY: `self.boxed` keeps the buffer until it is released
                sink.attach(|ptr, len| unsafe { d::JxlDecoderSetBoxBuffer(dec, ptr, len) })?;
                return Ok(None);
            }
            s::BoxComplete => {
                let Some(sink) = self.boxed.take() else {
                    return Ok(None);
                };
                // SAFETY: the decoder is valid while the session borrows it
                let remaining = unsafe { d::JxlDecoderReleaseBoxBuffer(dec) };
                Event::BoxComplete(sink.finish(remaining))
            }
        }))
    }

    fn release_box(&mut self) {
        if self.boxed.is_some() {
            // SAFETY: the decoder is valid while the session borrows it
            unsafe { d::JxlDecoderReleaseBoxBuffer(self.dec.dec) };
            self.boxed = None;
        }
    }

    fn frame_info(&self) -> eros::Result<FrameInfo, (GenericError,)> {
        let dec = self.dec.dec;
        let mut header = MaybeUninit::<JxlFrameHeader>::uninit();
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status(unsafe { d::JxlDecoderGetFrameHeader(dec, header.as_mut_ptr()) })?;
        // SAFETY: `GetFrameHeader` succeeded
        let header = unsafe { header.assume_init() };

        let mut name = vec![0u8; header.name_length as usize + 1];
        // SAFETY: the decoder is valid while the session borrows it, and `name` has room for the NUL
        check_dec_status(unsafe {
            d::JxlDecoderGetFrameName(dec, name.as_mut_ptr().cast(), name.len())
        })?;
        name.truncate(header.name_length as usize);

        Ok(FrameInfo {
            header,
            name: String::from_utf8_lossy(&name).into_owned(),
        })
    }

    fn box_header(&self) -> eros::Result<BoxHeader, (GenericError,)> {
        let dec = self.dec.dec;
        let mut box_type = JxlBoxType([0; 4]);
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status(unsafe {
            d::JxlDecoderGetBoxType(
                dec,
                &mut box_type,
                self.dec.decompress.unwrap_or(false).into(),
            )
        })?;

        let (mut raw, mut contents) = (0, 0);
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status(unsafe { d::JxlDecoderGetBoxSizeRaw(dec, &raw mut raw) })?;
        // SAFETY: the decoder is valid while the session borrows it
        let has_contents = unsafe { d::JxlDecoderGetBoxSizeContents(dec, &raw mut contents) }
            == d::JxlDecoderStatus::Success;

        Ok(BoxHeader {
            box_type: box_type.0.map(|c| c.to_ne_bytes()[0]),
            size_raw: raw,
            size_contents: has_contents.then_some(contents),
        })
    }

    /// ICC profile of the image, after [`Event::ColorEncoding`]
    ///
    /// # Errors
    /// Return [`NotAvailableYet`] before [`Event::ColorEncoding`], or [`GenericError`] if the
    /// profile is not available
    pub fn icc_profile(
        &self,
        target: ColorProfileTarget,
    ) -> eros::Result<Vec<u8>, (GenericError, NotAvailableYet)> {
        let dec = self.dec.dec;
        let mut size = 0;
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status(unsafe { d::JxlDecoderGetICCProfileSize(dec, target, &raw mut size) })?;

        let mut icc = vec![0; size];
        // SAFETY: the decoder is valid while the session borrows it, and `icc` holds `size` bytes
        check_dec_status(unsafe {
            d::JxlDecoderGetColorAsICCProfile(dec, target, icc.as_mut_ptr(), size)
        })?;
        Ok(icc)
    }

    /// Color encoding of the image, after [`Event::ColorEncoding`]
    ///
    /// # Errors
    /// Return [`NotAvailableYet`] before [`Event::ColorEncoding`], or [`GenericError`] if the
    /// image has no structured encoding, e.g. it uses an ICC profile
    pub fn color_encoding(
        &self,
        target: ColorProfileTarget,
    ) -> eros::Result<JxlColorEncoding, (GenericError, NotAvailableYet)> {
        let mut encoding = MaybeUninit::uninit();
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status(unsafe {
            d::JxlDecoderGetColorAsEncodedProfile(self.dec.dec, target, encoding.as_mut_ptr())
        })?;
        // SAFETY: `GetColorAsEncodedProfile` succeeded
        Ok(unsafe { encoding.assume_init() })
    }

    fn resolve_format(
        &self,
        format: PixelFormat,
        data_type: Option<JxlDataType>,
    ) -> eros::Result<JxlPixelFormat, (InvalidState, UnsupportedBitWidth)> {
        let info = self
            .basic_info
            .as_ref()
            .ok_or(InvalidState("basic info is not available yet"))
            .union()?;

        let data_type = match data_type {
            Some(v) => v,
            None => data_type_of(info.bits_per_sample, info.exponent_bits_per_sample).union()?,
        };

        Ok(JxlPixelFormat {
            num_channels: if format.num_channels == 0 {
                info.num_color_channels + u32::from(info.alpha_bits > 0)
            } else {
                format.num_channels
            },
            data_type,
            endianness: format.endianness,
            align: format.align,
        })
    }

    /// Provide a zeroed buffer for the frame after [`Event::NeedImageOutBuffer`].
    /// `data_type` is derived from the image if `None`.
    ///
    /// # Errors
    /// Return [`InvalidState`] before [`Event::BasicInfo`], [`UnsupportedBitWidth`] if the
    /// image has no matching pixel type, or [`GenericError`] if `libjxl` rejects the format
    pub fn alloc_image_buffer(
        &mut self,
        format: PixelFormat,
        data_type: Option<JxlDataType>,
    ) -> eros::Result<
        (),
        (
            InvalidState,
            UnsupportedBitWidth,
            GenericError,
            NotAvailableYet,
        ),
    > {
        let format = self.resolve_format(format, data_type).widen()?;
        let mut size = 0;
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status::<(GenericError, NotAvailableYet), _, _>(unsafe {
            d::JxlDecoderImageOutBufferSize(self.dec.dec, &raw const format, &raw mut size)
        })
        .widen()?;
        self.set_image_buffer(format, vec![0; size]).widen()
    }

    /// Provide the buffer for the frame after [`Event::NeedImageOutBuffer`].
    /// The session keeps it until [`Event::FullImage`] hands it back.
    ///
    /// # Errors
    /// Return [`NotAvailableYet`] before [`Event::BasicInfo`], or [`GenericError`] if the
    /// buffer is too small for the image
    pub fn set_image_buffer(
        &mut self,
        format: JxlPixelFormat,
        mut data: Vec<u8>,
    ) -> eros::Result<(), (GenericError, NotAvailableYet)> {
        // SAFETY: `self.image` keeps `data` until libjxl is done with it
        check_dec_status(unsafe {
            d::JxlDecoderSetImageOutBuffer(
                self.dec.dec,
                &raw const format,
                data.as_mut_ptr().cast(),
                data.len(),
            )
        })?;
        // The previous buffer is only dropped once libjxl points at the new one
        self.image = Some(Slot { format, data });
        Ok(())
    }

    /// Pixels decoded so far, to read after [`flush_image`](Self::flush_image)
    #[must_use]
    pub fn image_buffer(&self) -> Option<&[u8]> {
        self.image.as_ref().map(|s| s.data.as_slice())
    }

    /// Render the part of the frame that is decoded so far into the image buffer.
    /// Meant for [`Event::FrameProgression`] and truncated input.
    ///
    /// # Errors
    /// Return [`InvalidState`] if no image buffer is set, or [`GenericError`] if nothing is
    /// decoded yet
    pub fn flush_image(&mut self) -> eros::Result<(), (InvalidState, GenericError)> {
        if self.image.is_none() {
            return Err(InvalidState("no image buffer is set")).union();
        }
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status::<(GenericError,), _, _>(unsafe { d::JxlDecoderFlushImage(self.dec.dec) })
            .widen()
    }

    /// Provide a zeroed buffer for the preview after [`Event::NeedPreviewOutBuffer`]
    ///
    /// # Errors
    /// Same as [`alloc_image_buffer`](Self::alloc_image_buffer)
    pub fn alloc_preview_buffer(
        &mut self,
        format: PixelFormat,
        data_type: Option<JxlDataType>,
    ) -> eros::Result<
        (),
        (
            InvalidState,
            UnsupportedBitWidth,
            GenericError,
            NotAvailableYet,
        ),
    > {
        let format = self.resolve_format(format, data_type).widen()?;
        let mut size = 0;
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status::<(GenericError, NotAvailableYet), _, _>(unsafe {
            d::JxlDecoderPreviewOutBufferSize(self.dec.dec, &raw const format, &raw mut size)
        })
        .widen()?;

        let mut data = vec![0; size];
        // SAFETY: `self.preview` keeps `data` until libjxl is done with it
        check_dec_status::<(GenericError, NotAvailableYet), _, _>(unsafe {
            d::JxlDecoderSetPreviewOutBuffer(
                self.dec.dec,
                &raw const format,
                data.as_mut_ptr().cast(),
                data.len(),
            )
        })
        .widen()?;
        self.preview = Some(Slot { format, data });
        Ok(())
    }

    /// Collect the contents of the box announced by [`Event::Box`]. The buffer
    /// starts with `size` bytes and grows as needed. Without it the box is skipped.
    ///
    /// # Errors
    /// Return [`InvalidState`] without [`Events::BOX`], or [`GenericError`] if it is not
    /// called right after [`Event::Box`]
    pub fn set_box_buffer(
        &mut self,
        size: usize,
    ) -> eros::Result<(), (InvalidState, GenericError)> {
        if !self.events.contains(Events::BOX) {
            return Err(InvalidState("not subscribed to boxes")).union();
        }
        let dec = self.dec.dec;
        self.release_box();
        let mut sink = Sink::new(size);
        // SAFETY: `self.boxed` keeps the buffer until it is released
        sink.attach(|ptr, len| unsafe { d::JxlDecoderSetBoxBuffer(dec, ptr, len) })?;
        self.boxed = Some(sink);
        Ok(())
    }

    /// Description of an extra channel such as depth or a spot color
    ///
    /// # Errors
    /// Return [`NotAvailableYet`] before [`Event::BasicInfo`], or [`GenericError`] if `index`
    /// is out of range
    pub fn extra_channel_info(
        &self,
        index: usize,
    ) -> eros::Result<JxlExtraChannelInfo, (GenericError, NotAvailableYet)> {
        let mut info = MaybeUninit::uninit();
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status(unsafe {
            d::JxlDecoderGetExtraChannelInfo(self.dec.dec, index, info.as_mut_ptr())
        })?;
        // SAFETY: `GetExtraChannelInfo` succeeded
        Ok(unsafe { info.assume_init() })
    }

    /// Provide a zeroed buffer for extra channel `index` of the current frame, after
    /// [`Event::Frame`] or [`Event::NeedImageOutBuffer`]. Call it once per wanted channel
    /// and collect the pixels with [`take_extra_channel`](Self::take_extra_channel) after
    /// [`Event::FullImage`]. `data_type` is derived from the channel if `None`;
    /// `format.num_channels` is ignored.
    ///
    /// # Errors
    /// Return a [`GenericError`] if `index` is out of range or the format is not supported,
    /// [`NotAvailableYet`] before [`Event::BasicInfo`],
    /// or [`UnsupportedBitWidth`] if the channel has no matching pixel type
    pub fn alloc_extra_channel_buffer(
        &mut self,
        index: u32,
        format: PixelFormat,
        data_type: Option<JxlDataType>,
    ) -> eros::Result<(), (UnsupportedBitWidth, GenericError, NotAvailableYet)> {
        let data_type = if let Some(v) = data_type {
            v
        } else {
            let info = self.extra_channel_info(index as usize).widen()?;
            data_type_of(info.bits_per_sample, info.exponent_bits_per_sample).union()?
        };
        let format = JxlPixelFormat {
            num_channels: 1,
            data_type,
            endianness: format.endianness,
            align: format.align,
        };

        let mut size = 0;
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status::<(GenericError, NotAvailableYet), _, _>(unsafe {
            d::JxlDecoderExtraChannelBufferSize(
                self.dec.dec,
                &raw const format,
                &raw mut size,
                index,
            )
        })
        .widen()?;

        if self.extra_ready {
            self.extra.clear();
            self.extra_ready = false;
        }
        let mut data = vec![0; size];
        // SAFETY: `self.extra` keeps `data` until libjxl is done with it
        check_dec_status::<(GenericError, NotAvailableYet), _, _>(unsafe {
            d::JxlDecoderSetExtraChannelBuffer(
                self.dec.dec,
                &raw const format,
                data.as_mut_ptr().cast(),
                data.len(),
                index,
            )
        })
        .widen()?;
        self.extra.retain(|(i, _)| *i != index);
        self.extra.push((index, Slot { format, data }));
        Ok(())
    }

    /// Pixels of extra channel `index`, after [`Event::FullImage`]
    ///
    /// # Errors
    /// Return [`InvalidState`] if the frame is not fully decoded yet
    pub fn take_extra_channel(
        &mut self,
        index: u32,
    ) -> eros::Result<Option<Image>, (InvalidState,)> {
        if !self.extra_ready {
            return Err(InvalidState("the frame is not fully decoded").into());
        }
        Ok(self
            .extra
            .iter()
            .position(|(i, _)| *i == index)
            .map(|pos| self.extra.swap_remove(pos).1.into_image()))
    }

    /// Skip the next `amount` frames. Their events are not emitted.
    pub fn skip_frames(&mut self, amount: usize) {
        // SAFETY: the decoder is valid while the session borrows it
        unsafe { d::JxlDecoderSkipFrames(self.dec.dec, amount) };
    }

    /// Skip the rest of the frame being decoded. Valid after [`Event::NeedImageOutBuffer`]
    /// and before [`Event::FullImage`]; use [`flush_image`](Self::flush_image) first if the
    /// partial pixels are needed. The next event is the next [`Event::Frame`] or [`Event::Success`].
    ///
    /// # Errors
    /// Return a [`GenericError`] if no frame is being decoded
    pub fn skip_current_frame(&mut self) -> eros::Result<(), (GenericError,)> {
        // SAFETY: the decoder is valid while the session borrows it
        check_dec_status(unsafe { d::JxlDecoderSkipCurrentFrame(self.dec.dec) })
    }
}

impl Drop for Session<'_, '_, '_> {
    fn drop(&mut self) {
        // SAFETY: the decoder is valid while the session borrows it. The reset makes libjxl
        // forget the buffers before the fields owning them are dropped
        unsafe { d::JxlDecoderReset(self.dec.dec) }
    }
}

#[cfg(test)]
mod tests {
    use jpegxl_sys::metadata::codestream_header::JxlExtraChannelType as ExtraChannelType;
    use testresult::TestResult;

    use crate::{decoder_builder, tests::SAMPLE_JXL};

    use super::*;

    /// Kind of every event except `NeedMoreInput`, with the pixels of each frame
    fn decode_pieces<'a>(
        pieces: impl IntoIterator<Item = &'a [u8]>,
    ) -> TestResult<Vec<(std::mem::Discriminant<Event>, Vec<u8>)>> {
        let mut decoder = decoder_builder().build()?;
        let mut session = decoder.session(
            Events::COLOR_ENCODING | Events::FRAME | Events::FRAME_PROGRESSION | Events::FULL_IMAGE,
        )?;
        let mut pieces = pieces.into_iter();
        let mut input: &[u8] = &[];
        let mut seen = vec![];
        loop {
            let event = session.process(&mut input)?;
            match &event {
                Event::NeedMoreInput => {
                    input = pieces.next().expect("input exhausted");
                    continue;
                }
                Event::NeedImageOutBuffer => {
                    session.alloc_image_buffer(PixelFormat::default(), None)?;
                }
                Event::Success => return Ok(seen),
                _ => {}
            }
            let pixels = match &event {
                Event::FullImage(Some(image)) => image.data.clone(),
                _ => vec![],
            };
            seen.push((std::mem::discriminant(&event), pixels));
        }
    }

    #[test]
    fn chunked_input_matches_whole() -> TestResult {
        use crate::tests::{SAMPLE_BOXES, SAMPLE_JXL_JPEG};
        for data in [SAMPLE_JXL, SAMPLE_BOXES, SAMPLE_JXL_JPEG] {
            let whole = decode_pieces([data])?;
            for chunk in [1, 7, 20, 100, 1000] {
                assert_eq!(
                    decode_pieces(data.chunks(chunk))?,
                    whole,
                    "chunk size {chunk}"
                );
            }
            // Some of these split a section that is larger than the bytes glued to the carry
            for at in [1, 38, 51, 200] {
                let (head, tail) = data.split_at(at);
                assert_eq!(decode_pieces([head, tail])?, whole, "split at {at}");
            }
        }
        Ok(())
    }

    #[test]
    fn data_type_follows_bit_depth() {
        assert_eq!(data_type_of(1, 0).ok(), Some(JxlDataType::Uint8));
        assert_eq!(data_type_of(12, 0).ok(), Some(JxlDataType::Uint16));
        assert_eq!(data_type_of(16, 5).ok(), Some(JxlDataType::Float16));
        assert_eq!(data_type_of(32, 8).ok(), Some(JxlDataType::Float));
        assert_eq!(data_type_of(24, 0), Err(UnsupportedBitWidth(24)));
    }

    #[test]
    fn color_encoding_and_missing_preview() -> TestResult {
        let mut decoder = decoder_builder().build()?;
        let mut session = decoder.session(Events::COLOR_ENCODING)?;
        let error = session
            .alloc_preview_buffer(PixelFormat::default(), None)
            .unwrap_err();
        assert!(error.narrow::<InvalidState, _>().is_ok());
        let mut input = SAMPLE_JXL;
        loop {
            match session.process(&mut input)? {
                Event::BasicInfo(info) => {
                    assert_eq!(info.have_preview, jpegxl_sys::common::types::JxlBool::False);
                    assert!(session
                        .alloc_preview_buffer(PixelFormat::default(), None)
                        .is_err());
                }
                Event::ColorEncoding => {
                    let encoding = session.color_encoding(ColorProfileTarget::Data)?;
                    let srgb: JxlColorEncoding = (&crate::encode::ColorEncoding::Srgb).into();
                    assert_eq!(encoding.white_point, srgb.white_point);
                    assert_eq!(encoding.primaries, srgb.primaries);
                    return Ok(());
                }
                Event::Success => panic!("no color encoding"),
                _ => {}
            }
        }
    }

    const BENCH_JXL: &[u8] = include_bytes!("../../../samples/bench.jxl");

    fn flush_truncated(len: usize) -> TestResult<bool> {
        let decoder = &mut decoder_builder().build()?;
        let mut session = decoder.session(Events::FULL_IMAGE)?;
        let mut input = &BENCH_JXL[..len];
        loop {
            match session.process(&mut input)? {
                Event::NeedImageOutBuffer => {
                    session.alloc_image_buffer(PixelFormat::default(), None)?;
                }
                Event::NeedMoreInput => {
                    return Ok(session.flush_image().is_ok()
                        && session
                            .image_buffer()
                            .is_some_and(|b| b.iter().any(|&x| x != 0)));
                }
                Event::Success => panic!("truncated input decoded completely"),
                _ => {}
            }
        }
    }

    #[test]
    fn truncated_input_flushes_partial_image() -> TestResult {
        let flushed = [10, 15]
            .into_iter()
            .map(|i| flush_truncated(BENCH_JXL.len() * i / 20))
            .collect::<Result<Vec<_>, _>>()?;
        assert!(flushed.contains(&true), "{flushed:?}");
        Ok(())
    }

    #[test]
    fn boxes_are_collected() -> TestResult {
        let data = crate::tests::SAMPLE_BOXES;
        let mut decoder = decoder_builder().decompress(true).build()?;
        let mut session = decoder.session(Events::BOX)?;
        let mut input = data;
        let mut boxes = vec![];
        let mut current = None;
        session.close_input();
        loop {
            match session.process(&mut input)? {
                Event::Box(header) if [*b"Exif", *b"xml "].contains(&header.box_type) => {
                    current = Some(header.box_type);
                    // Setting the buffer again replaces the first one
                    session.set_box_buffer(1024)?;
                    session.set_box_buffer(1)?;
                }
                Event::BoxComplete(contents) => boxes.push((current.take(), contents)),
                Event::Success => break,
                _ => {}
            }
        }

        assert_eq!(boxes.len(), 2);
        let find = |t: &[u8; 4]| boxes.iter().find(|(ty, _)| ty.as_ref() == Some(t));
        assert!(find(b"Exif").is_some_and(|(_, c)| c.ends_with(crate::tests::SAMPLE_EXIF)));
        assert_eq!(
            find(b"xml ").map(|(_, c)| c.as_slice()),
            Some(crate::tests::SAMPLE_XMP)
        );
        Ok(())
    }

    #[test]
    fn jpeg_is_reconstructed_bit_exact() -> TestResult {
        let mut decoder = decoder_builder().init_jpeg_buffer(16).build()?;
        let mut session = decoder.session(Events::JPEG_RECONSTRUCTION | Events::FULL_IMAGE)?;
        let mut input = crate::tests::SAMPLE_JXL_JPEG;
        loop {
            match session.process(&mut input)? {
                Event::Jpeg(jpeg) => {
                    assert_eq!(jpeg, crate::tests::SAMPLE_JPEG);
                    return Ok(());
                }
                Event::Success => panic!("no JPEG reconstructed"),
                _ => {}
            }
        }
    }

    #[test]
    fn close_input_in_chunks() -> TestResult {
        let mut decoder = decoder_builder().build()?;
        let mut session = decoder.session(Events::FULL_IMAGE | Events::COLOR_ENCODING)?;
        let mut input = SAMPLE_JXL;
        session.close_input();
        let mut frames = 0;
        loop {
            match session.process(&mut input)? {
                Event::ColorEncoding => {
                    assert_ne!(session.icc_profile(ColorProfileTarget::Data)?.len(), 0);
                }
                Event::NeedImageOutBuffer => {
                    session.alloc_image_buffer(PixelFormat::default(), None)?;
                }
                Event::FullImage(Some(_)) => frames += 1,
                Event::Success => break,
                _ => {}
            }
        }
        assert_eq!(frames, 1);
        Ok(())
    }

    #[test]
    fn closed_truncated_input_is_an_error() -> TestResult {
        let mut decoder = decoder_builder().build()?;
        let mut session = decoder.session(Events::FULL_IMAGE)?;
        let mut input = &SAMPLE_JXL[..SAMPLE_JXL.len() / 2];
        session.close_input();
        loop {
            match session.process(&mut input) {
                Ok(Event::NeedImageOutBuffer) => {
                    session.alloc_image_buffer(PixelFormat::default(), None)?;
                }
                Ok(Event::Success) => panic!("truncated input decoded completely"),
                Ok(_) => {}
                Err(e) => {
                    assert!(e.narrow::<(GenericError, IncompleteInput), _>().is_ok());
                    return Ok(());
                }
            }
        }
    }

    /// (duration, name, `is_last`, pixels) of every frame, after running `on_event`
    #[allow(clippy::type_complexity)]
    fn frames_of(
        data: &[u8],
        mut on_event: impl FnMut(&mut Session, &Event) -> TestResult<bool>,
    ) -> TestResult<Vec<(u32, String, bool, Vec<u8>)>> {
        let mut decoder = decoder_builder().build()?;
        let mut session = decoder.session(Events::FRAME | Events::FULL_IMAGE)?;
        let mut input = data;
        let (mut frames, mut current) = (vec![], None);
        loop {
            let event = session.process(&mut input)?;
            if on_event(&mut session, &event)? {
                continue;
            }
            match event {
                Event::Frame(f) => {
                    current = Some((
                        f.header.duration,
                        f.name,
                        f.header.is_last == jpegxl_sys::common::types::JxlBool::True,
                    ));
                }
                Event::NeedImageOutBuffer => {
                    session.alloc_image_buffer(PixelFormat::default(), None)?;
                }
                Event::FullImage(Some(image)) => {
                    let (duration, name, is_last) = current.take().expect("frame without header");
                    frames.push((duration, name, is_last, image.data));
                }
                Event::Success => return Ok(frames),
                _ => {}
            }
        }
    }

    #[test]
    fn animation_frames() -> TestResult {
        let data = crate::tests::SAMPLE_ANIMATION;
        let frames = frames_of(data, |_, _| Ok(false))?;

        let summary: Vec<_> = frames.iter().map(|f| (f.0, f.1.as_str(), f.2)).collect();
        assert_eq!(
            summary,
            [
                (10, "frame0", false),
                (20, "frame1", false),
                (30, "frame2", true)
            ]
        );
        assert_ne!(frames[0].3, frames[1].3);
        assert_ne!(frames[1].3, frames[2].3);

        let (_, one_shot) = decoder_builder().build()?.decode_with::<u8>(data)?;
        assert_eq!(
            one_shot, frames[2].3,
            "one-shot decode returns the last frame"
        );
        Ok(())
    }

    #[test]
    fn skip_frames_and_current_frame() -> TestResult {
        let data = crate::tests::SAMPLE_ANIMATION;
        let all = frames_of(data, |_, _| Ok(false))?;

        let skipped = frames_of(data, |s, e| {
            if matches!(e, Event::BasicInfo(_)) {
                s.skip_frames(1);
            }
            Ok(false)
        })?;
        let durations: Vec<_> = skipped.iter().map(|f| f.0).collect();
        assert_eq!(durations, [20, 30]);
        assert_eq!(skipped[0].3, all[1].3);

        let mut first = true;
        let skipped = frames_of(data, |s, e| {
            if matches!(e, Event::NeedImageOutBuffer) && std::mem::take(&mut first) {
                s.alloc_image_buffer(PixelFormat::default(), None)?;
                s.skip_current_frame()?;
                return Ok(true);
            }
            Ok(false)
        })?;
        let durations: Vec<_> = skipped.iter().map(|f| f.0).collect();
        assert_eq!(durations, [20, 30]);
        Ok(())
    }

    #[test]
    fn extra_channel_of_every_layer() -> TestResult {
        let pixels = vec![200u8; 8 * 8 * 4];
        let frame = crate::encode::EncoderFrame::new(&pixels).num_channels(4);
        let info = crate::encode::ImageInfo::builder()
            .width(8)
            .height(8)
            .has_alpha(true)
            .build();
        let mut encoder = crate::encoder_builder().build()?;
        let mut enc = encoder.session(&info)?;
        enc.add_frame(&frame)?;
        enc.add_frame(&frame)?;
        let data = enc.finish()?;

        let mut decoder = decoder_builder().coalescing(false).build()?;
        let mut session = decoder.session(Events::FULL_IMAGE)?;
        let mut input = data.as_slice();
        let mut layers = 0;
        loop {
            match session.process(&mut input)? {
                Event::NeedImageOutBuffer => {
                    session.alloc_image_buffer(PixelFormat::default(), None)?;
                    let format = PixelFormat::default();
                    session.alloc_extra_channel_buffer(0, format, Some(JxlDataType::Uint16))?;
                }
                Event::FullImage(_) => {
                    let alpha = session.take_extra_channel(0)?.expect("no alpha channel");
                    assert_eq!(alpha.format.data_type, JxlDataType::Uint16);
                    layers += 1;
                }
                Event::Success => break,
                _ => {}
            }
        }
        assert_eq!(layers, 2);
        Ok(())
    }

    #[test]
    fn layers_without_coalescing() -> TestResult {
        let data = crate::tests::SAMPLE_LAYERS;
        let mut decoder = decoder_builder().coalescing(false).build()?;
        let mut session = decoder.session(Events::FRAME)?;
        let mut input = data;
        let mut frames = 0;
        loop {
            match session.process(&mut input)? {
                Event::Frame(_) => frames += 1,
                Event::Success => break,
                _ => {}
            }
        }
        assert_eq!(frames, 3);
        Ok(())
    }

    #[test]
    fn progressive_steps_flush() -> TestResult {
        let data = crate::tests::SAMPLE_PROGRESSIVE;
        let mut decoder = decoder_builder()
            .progressive_detail(super::super::ProgressiveDetail::Passes)
            .build()?;
        let mut session = decoder.session(Events::FRAME_PROGRESSION | Events::FULL_IMAGE)?;
        let mut input = data;
        let mut steps = 0;
        loop {
            match session.process(&mut input)? {
                Event::NeedImageOutBuffer => {
                    session.alloc_image_buffer(PixelFormat::default(), None)?;
                }
                Event::FrameProgression => {
                    session.flush_image()?;
                    assert!(session
                        .image_buffer()
                        .is_some_and(|b| b.iter().any(|&x| x != 0)));
                    steps += 1;
                }
                Event::Success => break,
                _ => {}
            }
        }
        assert!(steps >= 2, "{steps} progressive steps");
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn extra_channel_is_collected() -> TestResult {
        let mut decoder = decoder_builder().build()?;
        let mut session = decoder.session(Events::FULL_IMAGE)?;
        let mut input = crate::tests::SAMPLE_EXTRA_CHANNEL;
        loop {
            match session.process(&mut input)? {
                Event::BasicInfo(info) => assert_eq!(info.num_extra_channels, 1),
                Event::NeedImageOutBuffer => {
                    let info = session.extra_channel_info(0)?;
                    assert_eq!(info.r#type, ExtraChannelType::Depth);
                    session.alloc_image_buffer(PixelFormat::default(), None)?;
                    session.alloc_extra_channel_buffer(0, PixelFormat::default(), None)?;
                    let error = session.take_extra_channel(0).unwrap_err();
                    assert!(error.narrow::<InvalidState, _>().is_ok());
                }
                Event::FullImage(_) => {
                    let depth = session.take_extra_channel(0)?.expect("no depth channel");
                    let expected: Vec<u8> =
                        (0..128u32 * 128).map(|i| (i % 128 * 2) as u8).collect();
                    assert_eq!(depth.data, expected);
                    assert!(session.take_extra_channel(0)?.is_none());
                }
                Event::Success => return Ok(()),
                _ => {}
            }
        }
    }

    #[test]
    fn decoder_is_reusable_after_error() -> TestResult {
        let decoder = decoder_builder().build()?;
        assert!(decoder.decode(&SAMPLE_JXL[..100]).is_err());
        decoder.decode(SAMPLE_JXL)?;
        Ok(())
    }

    #[test]
    fn misuse_is_an_error() -> TestResult {
        let mut decoder = decoder_builder().build()?;
        let mut session = decoder.session(Events::empty())?;
        let error = session
            .alloc_image_buffer(PixelFormat::default(), None)
            .unwrap_err();
        assert!(error.narrow::<InvalidState, _>().is_ok());
        let error = session.flush_image().unwrap_err();
        assert!(error.narrow::<InvalidState, _>().is_ok());
        let error = session.set_box_buffer(8).unwrap_err();
        assert!(error.narrow::<InvalidState, _>().is_ok());
        Ok(())
    }
}
