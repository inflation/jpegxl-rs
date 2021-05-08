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
//! use jpegxl_rs::{decoder_builder, memory::MallocManager};
//! let mut manager = MallocManager::default();
//! let mut decoder = decoder_builder().memory_manager(&manager).build()?;
//! # Ok(())
//! # };
//! ```

use std::{
    alloc::{GlobalAlloc as _, Layout, System},
    collections::HashMap,
    ffi::c_void,
    ptr::null_mut,
};

/// Allocator function type
pub type AllocFn = unsafe extern "C" fn(opaque: *mut c_void, size: usize) -> *mut c_void;
/// Deallocator function type
pub type FreeFn = unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void);

/// General trait for a memory manager

pub trait JxlMemoryManager {
    /// Return a custom allocator function. Can be None for using default one
    fn alloc(&self) -> Option<AllocFn>;
    /// Return a custom deallocator function. Can be None for using default one
    fn free(&self) -> Option<FreeFn>;

    /// Helper conversion function for C API
    fn to_manager(&self) -> jpegxl_sys::JxlMemoryManager {
        jpegxl_sys::JxlMemoryManager {
            opaque: (self as *const _ as *mut Self).cast(),
            alloc: self.alloc(),
            free: self.free(),
        }
    }
}

/// Example implement of [`JxlMemoryManager`]
#[derive(Debug)]
pub struct MallocManager {
    layouts: HashMap<*mut c_void, Layout>,
}

impl Default for MallocManager {
    fn default() -> Self {
        Self {
            layouts: HashMap::new(),
        }
    }
}

impl JxlMemoryManager for MallocManager {
    fn alloc(&self) -> Option<AllocFn> {
        unsafe extern "C" fn alloc(opaque: *mut c_void, size: usize) -> *mut c_void {
            match Layout::from_size_align(size, 8) {
                Ok(layout) => {
                    let address = System.alloc(layout);

                    let manager = &mut *opaque.cast::<MallocManager>();
                    manager.layouts.insert(address.cast(), layout);

                    address.cast()
                }
                _ => null_mut(),
            }
        }

        Some(alloc)
    }

    fn free(&self) -> Option<FreeFn> {
        unsafe extern "C" fn free(opaque: *mut c_void, address: *mut c_void) {
            if let Some(layout) = (*opaque.cast::<MallocManager>()).layouts.remove(&address) {
                System.dealloc(address.cast(), layout);
            }
        }

        Some(free)
    }
}
