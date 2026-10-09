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
//! Functions return an [`eros::ErrorUnion`] of the kinds of failure they can produce, e.g.
//! `eros::Result<_, (InvalidState, GenericError)>`. Each kind is one type: `libjxl` failures
//! are [`GenericError`](crate::decode::GenericError) and
//! [`EncoderFailure`](crate::encode::EncoderFailure), and the others are errors of this
//! crate. They live next to the API that returns them, in [`decode`](crate::decode) and
//! [`encode`](crate::encode), and the shared ones here. A failure that cannot happen in a
//! function's flow is reported as [`InternalError`] instead of being passed on.
//!
//! A union converts into a wider one with [`widen`](eros::ErrorUnion::widen), and a failure
//! or a group of them is picked out with [`narrow`](eros::ErrorUnion::narrow).

use eros::{type_set::Contains, ErrorUnion, TypeSet};
use thiserror::Error;

/// A session method is called at the wrong time
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Invalid session state: {0}")]
pub struct InvalidState(pub(crate) &'static str);

/// A bug in this crate, e.g. a failure that cannot happen in the flow of the function
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Internal error, please file an issue: {0}")]
pub struct InternalError(pub(crate) &'static str);

/// Handler for [`try_recover`](eros::ReshapeUnion::try_recover) of failures that the flow of
/// the caller rules out. They are reported as an [`InternalError`], with the original failure
/// as context. No test can reach it
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) fn ruled_out<E, T, S, I>(failure: ErrorUnion<E>) -> eros::Result<T, S>
where
    E: TypeSet,
    S: TypeSet,
    S::Variants: Contains<InternalError, I>,
{
    let error: ErrorUnion<S> = ErrorUnion::new(InternalError("a failure that the flow rules out"));
    Err(error.context(failure.into_inner()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ruled_out_reports_a_bug_with_the_failure() {
        let failure: ErrorUnion<(InvalidState,)> = ErrorUnion::new(InvalidState("taken"));
        let result: eros::Result<(), (InternalError,)> = ruled_out(failure);
        let error = result.unwrap_err();
        let context: Vec<_> = error.contexts().map(ToString::to_string).collect();
        assert_eq!(context, ["Invalid session state: taken"]);
        assert_eq!(*error, InternalError("a failure that the flow rules out"));
    }
}
