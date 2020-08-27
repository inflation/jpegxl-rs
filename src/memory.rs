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
#![deny(missing_docs)]

use jpegxl_sys::*;
use std::ffi::c_void;

pub trait JpegxlMemoryManager {
    unsafe extern "C" fn alloc(opaque: *mut c_void, size: size_t) -> *mut c_void;
    unsafe extern "C" fn free(opaque: *mut c_void, address: *mut c_void);

    fn to_manager(&mut self) -> JpegxlMemoryManagerStruct {
        JpegxlMemoryManagerStruct {
            opaque: self.as_opaque_ptr(),
            alloc: Some(Self::alloc),
            free: Some(Self::free),
        }
    }

    fn as_opaque_ptr(&mut self) -> *mut c_void {
        self as *mut Self as *mut c_void
    }
}
