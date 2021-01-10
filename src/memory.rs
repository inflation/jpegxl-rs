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

//! Memory manager interface
//! # Example
//! ```
//! # || -> Result<(), Box<dyn std::error::Error>> {
//!        use jpegxl_rs::memory::*;
//!        use std::alloc::{GlobalAlloc, Layout, System};
//!        use std::ffi::c_void;
//!
//!        #[derive(Debug)]
//!        struct MallocManager {
//!            layout: Layout,
//!        }
//!
//!        impl JXLMemoryManager for MallocManager {
//!            fn alloc(&self) -> Option<AllocFn> {
//!                unsafe extern "C" fn alloc(opaque: *mut c_void, size: size_t) -> *mut c_void {
//!                    println!("Custom alloc");
//!                    let layout = Layout::from_size_align(size as usize, 8).unwrap();
//!                    let address = System.alloc(layout);
//!
//!                    let manager = (opaque as *mut MallocManager).as_mut().unwrap();
//!                    manager.layout = layout;
//!
//!                    address as *mut c_void
//!                }
//!
//!                Some(alloc)
//!            }
//!
//!            fn free(&self) -> Option<FreeFn> {
//!                unsafe extern "C" fn free(opaque: *mut c_void, address: *mut c_void) {
//!                    println!("Custom dealloc");
//!                    let layout = (opaque as *mut MallocManager).as_mut().unwrap().layout;
//!                    System.dealloc(address as *mut u8, layout);
//!                }
//!
//!                Some(free)
//!            }
//!        }
//! # Ok(()) };
//! ```

use std::alloc::{GlobalAlloc, Layout, System};
use std::ffi::c_void;

pub use jpegxl_sys::size_t;
use jpegxl_sys::*;

/// Allocator function type
pub type AllocFn = unsafe extern "C" fn(opaque: *mut c_void, size: size_t) -> *mut c_void;
/// Deallocator function type
pub type FreeFn = unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void);

/// General trait for a memory manager

pub trait JXLMemoryManager: std::fmt::Debug {
    /// Return a custom allocator function. Can be None for using default one
    fn alloc(&self) -> Option<AllocFn>;
    /// Return a custom deallocator function. Can be None for using default one
    fn free(&self) -> Option<FreeFn>;

    /// Helper conversion function for C API
    fn to_manager(&mut self) -> JxlMemoryManager {
        JxlMemoryManager {
            opaque: self.as_opaque_ptr(),
            alloc: self.alloc(),
            free: self.free(),
        }
    }

    /// Helper function to get an opaque pointer
    fn as_opaque_ptr(&mut self) -> *mut c_void {
        self as *mut Self as *mut c_void
    }
}

/// Example implement of JXLMemoryManager
#[derive(Debug)]
pub(crate) struct MallocManager {
    layout: Layout,
}

impl Default for MallocManager {
    fn default() -> Self {
        Self {
            layout: Layout::from_size_align(0, 8).unwrap(),
        }
    }
}

impl JXLMemoryManager for MallocManager {
    fn alloc(&self) -> Option<AllocFn> {
        unsafe extern "C" fn alloc(opaque: *mut c_void, size: size_t) -> *mut c_void {
            let layout = Layout::from_size_align(size as usize, 8).unwrap();
            let address = System.alloc(layout);

            let manager = (opaque as *mut MallocManager).as_mut().unwrap();
            manager.layout = layout;

            address as *mut c_void
        }

        Some(alloc)
    }

    fn free(&self) -> Option<FreeFn> {
        unsafe extern "C" fn free(opaque: *mut c_void, address: *mut c_void) {
            let layout = (opaque as *mut MallocManager).as_mut().unwrap().layout;
            System.dealloc(address as *mut u8, layout);
        }

        Some(free)
    }
}
