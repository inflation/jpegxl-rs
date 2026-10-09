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
//! Every failure is its own small type: the shared ones live here, the others next to the
//! API that returns them, in [`decode`](crate::decode) and [`encode`](crate::encode).
//! Functions return an [`eros::ErrorUnion`] of exactly the failures they can produce, e.g.
//! `eros::Result<_, (InvalidState, GenericError)>`, so callers only handle what can
//! actually happen. The `libjxl` failures of each call are taken from the `libjxl`
//! sources. A union converts into a wider one with [`widen`](eros::ErrorUnion::widen),
//! and a failure or a group of them is picked out with [`narrow`](eros::ErrorUnion::narrow).

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

/// Internal error, usually invalid usages of the `libjxl` library
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error("Internal error, please file an issue: {0}")]
pub struct InternalError(pub(crate) &'static str);
