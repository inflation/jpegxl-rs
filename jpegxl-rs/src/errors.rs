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

//! Errors shared by the decoder and the encoder
//!
//! The session APIs, [`decode::Session`](crate::decode::Session) and
//! [`encode::Session`](crate::encode::Session), drive `libjxl` call by call, so they return
//! the `libjxl` failures of each call as individual types, e.g.
//! `eros::Result<_, (InvalidState, GenericError)>`. The one-shot functions, e.g.
//! [`JxlDecoder::decode`](crate::decode::JxlDecoder::decode), only list the failures a caller
//! can act on, and report everything else as one [`Failure`] that keeps the original error.
//!
//! A union converts into a wider one with [`widen`](eros::ErrorUnion::widen), and a failure
//! or a group of them is picked out with [`narrow`](eros::ErrorUnion::narrow).

use eros::{type_set::Contains, ErrorUnion, SendSyncError, TypeSet};
use thiserror::Error;

/// `libjxl` reported a generic error (`JXL_DEC_ERROR` or `JXL_ENC_ERR_GENERIC`)
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error(
    "Generic Error. Please build `libjxl` from source (using `vendored` feature) \
    in debug mode to get more information. Check `stderr` for any internal error messages."
)]
pub struct GenericError;

/// A session method is called at the wrong time
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Invalid session state: {0}")]
pub struct InvalidState(pub(crate) &'static str);

/// A bug in this crate, e.g. a failure that cannot happen in the flow of the function
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Internal error, please file an issue: {0}")]
pub struct InternalError(pub(crate) &'static str);

/// A failure of a one-shot function that the caller cannot act on: a `libjxl` error, a misuse
/// or a bug. The original error is its [`source`](std::error::Error::source)
#[derive(Error, Debug)]
#[error("JPEG XL operation failed")]
pub struct Failure(#[source] Box<dyn SendSyncError>);

impl Failure {
    /// The original error if it is a `T`, e.g. [`GenericError`]
    #[must_use]
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        // `Box<dyn SendSyncError>` is a `SendSyncError` too, so look inside the box
        (*self.0).as_any().downcast_ref()
    }
}

impl From<InternalError> for Failure {
    fn from(error: InternalError) -> Self {
        Self(Box::new(error))
    }
}

impl<E: TypeSet> From<ErrorUnion<E>> for Failure {
    fn from(error: ErrorUnion<E>) -> Self {
        Self(error.into_inner())
    }
}

/// Handler for [`try_recover`](eros::ReshapeUnion::try_recover) that reports the selected
/// failures as one [`Failure`]
pub(crate) fn into_failure<E, T, S, I>(failure: ErrorUnion<E>) -> eros::Result<T, S>
where
    E: TypeSet,
    S: TypeSet,
    S::Variants: Contains<Failure, I>,
{
    Err(ErrorUnion::new(Failure::from(failure)))
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn failure_keeps_the_original_error() {
        let error: ErrorUnion<(InvalidState, GenericError)> = ErrorUnion::new(GenericError);
        let failure = Failure::from(error);
        assert_eq!(failure.downcast_ref::<GenericError>(), Some(&GenericError));
        assert!(failure.downcast_ref::<InvalidState>().is_none());
        assert_eq!(
            failure.source().map(ToString::to_string),
            Some(GenericError.to_string())
        );
        println!("{failure}, {failure:?}");
    }
}
