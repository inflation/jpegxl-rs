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

use thiserror::Error;

/// A session method is called at the wrong time
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Invalid session state: {0}")]
pub struct InvalidState(pub(crate) &'static str);

/// A bug in this crate, e.g. a failure that cannot happen in the flow of the function
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Internal error, please file an issue: {0}")]
pub struct InternalError(pub(crate) &'static str);
