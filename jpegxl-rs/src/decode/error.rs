/*
This file is part of jpegxl-rs.

jpegxl-rs is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-rs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
*/

use eros::{
    type_set::{Contains, GroupNarrow, SupersetOf},
    ErrorUnion, TypeSet,
};
use jpegxl_sys::decode::JxlDecoderStatus;
use thiserror::Error;

use crate::errors::{GenericError, InternalError, InvalidState};

/// `libjxl` failed to create a decoder, usually because the memory manager is out of memory
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Cannot create a decoder")]
pub struct CannotCreateDecoder;

/// The input does not contain a valid codestream or container
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("The input does not contain a valid codestream or container")]
pub struct InvalidInput;

/// The input ended before the image was complete
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("The input ended before the image was complete")]
pub struct IncompleteInput;

/// The image uses a pixel bit width without a matching Rust type
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Unsupported Pixel bit width: {0}")]
pub struct UnsupportedBitWidth(pub u32);

/// The information is requested before `libjxl` decoded it (`JXL_DEC_NEED_MORE_INPUT` from a
/// getter). Call again after the event the method names
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("The information is not decoded yet")]
pub struct NotAvailableYet;

/// Everything a one-shot decode, e.g. [`JxlDecoder::decode`](super::JxlDecoder::decode),
/// can fail with
pub type DecodeErrors = (
    InvalidInput,
    IncompleteInput,
    GenericError,
    NotAvailableYet,
    UnsupportedBitWidth,
    InvalidState,
    InternalError,
);

/// Map a decoder status to the failures `S` of the call that returned it.
///
/// `JXL_DEC_NEED_MORE_INPUT` from a getter is [`NotAvailableYet`]. A status outside `S`
/// contradicts the `libjxl` sources, and becomes a [`GenericError`]
pub(crate) fn check_dec_status<S, I, J>(status: JxlDecoderStatus) -> eros::Result<(), S>
where
    S: TypeSet,
    S::Variants: Contains<GenericError, J>,
    <(GenericError, NotAvailableYet) as TypeSet>::Variants: SupersetOf<S::Variants, I>,
{
    let error: ErrorUnion<(GenericError, NotAvailableYet)> = match status {
        JxlDecoderStatus::Success => return Ok(()),
        JxlDecoderStatus::NeedMoreInput => ErrorUnion::new(NotAvailableYet),
        _ => ErrorUnion::new(GenericError),
    };
    Err(error
        .narrow::<S, GroupNarrow<I>>()
        .unwrap_or_else(|_| ErrorUnion::new(GenericError)))
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use crate::tests::failure;

    use super::*;

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn decode_invalid_data() -> TestResult {
        let decoder = crate::decoder_builder().build()?;
        assert!(failure::<InvalidInput, _, _>(&decoder.decode_with::<u8>(&[])).is_some());
        assert!(failure::<InvalidInput, _, _>(&decoder.decode_with::<u8>(&[0; 64])).is_some());
        assert!(failure::<IncompleteInput, _, _>(
            &decoder.decode(&crate::tests::SAMPLE_JXL[..100])
        )
        .is_some());
        Ok(())
    }

    #[test]
    fn statuses_map_to_the_failures_of_the_call() {
        let status: eros::Result<(), (GenericError, NotAvailableYet)> =
            check_dec_status(JxlDecoderStatus::Error);
        assert_eq!(
            *status.unwrap_err().narrow::<(GenericError,), _>().unwrap(),
            GenericError
        );

        let status: eros::Result<(), (GenericError, NotAvailableYet)> =
            check_dec_status(JxlDecoderStatus::NeedMoreInput);
        let error = status.unwrap_err();
        println!("{error}, {error:?}");
        assert!(error.narrow::<NotAvailableYet, _>().is_ok());

        // A status the call never returns folds into the generic error
        let status: eros::Result<(), (GenericError,)> =
            check_dec_status(JxlDecoderStatus::NeedMoreInput);
        assert_eq!(*status.unwrap_err(), GenericError);
    }
}
