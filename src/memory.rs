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
use std::ffi::c_void;

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

#[cfg(test)]
pub(crate) mod tests {
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        collections::HashMap,
        ptr::null_mut,
        sync::Mutex,
    };

    use super::*;

    pub(crate) struct NoManager {}

    impl JxlMemoryManager for NoManager {
        fn alloc(&self) -> Option<crate::memory::AllocFn> {
            unsafe extern "C" fn alloc(_a: *mut c_void, _b: usize) -> *mut c_void {
                null_mut()
            }

            Some(alloc)
        }

        fn free(&self) -> Option<crate::memory::FreeFn> {
            None
        }
    }

    /// Example implement of [`JxlMemoryManager`]
    pub struct MallocManager {
        layouts: Mutex<HashMap<*mut c_void, Layout>>,
    }

    impl Default for MallocManager {
        fn default() -> Self {
            Self {
                layouts: Mutex::new(HashMap::new()),
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
                        if let Ok(mut mg) = manager.layouts.lock() {
                            mg.insert(address.cast(), layout);
                        } else {
                            return null_mut();
                        }

                        address.cast()
                    }
                    Err(_) => null_mut(),
                }
            }

            Some(alloc)
        }

        fn free(&self) -> Option<FreeFn> {
            unsafe extern "C" fn free(opaque: *mut c_void, address: *mut c_void) {
                if let Ok(mg) = &mut (*opaque.cast::<MallocManager>()).layouts.lock() {
                    if let Some(layout) = mg.remove(&address) {
                        System.dealloc(address.cast(), layout);
                    }
                };
            }

            Some(free)
        }
    }

    #[test]
    fn test_no_manager() {
        use crate::{decoder_builder, encoder_builder, DecodeError, EncodeError};

        let memory_manager = NoManager {};

        let decoder = decoder_builder().memory_manager(&memory_manager).build();
        assert!(matches!(decoder, Err(DecodeError::CannotCreateDecoder)));

        let encoder = encoder_builder().memory_manager(&memory_manager).build();
        assert!(matches!(encoder, Err(EncodeError::CannotCreateEncoder)));
    }
}
