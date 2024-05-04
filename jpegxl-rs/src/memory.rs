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

use jpegxl_sys::memory_manager::JxlMemoryManager;

/// Allocating function type
pub type AllocFn = unsafe extern "C" fn(opaque: *mut c_void, size: usize) -> *mut c_void;
/// Deallocating function type
pub type FreeFn = unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void);

/// General trait for a memory manager

#[allow(clippy::module_name_repetitions)]
pub trait MemoryManager {
    /// Return a custom allocating function
    fn alloc(&self) -> AllocFn;
    /// Return a custom deallocating function
    fn free(&self) -> FreeFn;

    /// Helper conversion function for C API
    #[must_use]
    fn manager(&self) -> JxlMemoryManager {
        #[allow(clippy::ref_as_ptr)]
        JxlMemoryManager {
            opaque: (self as *const Self).cast_mut().cast(),
            alloc: self.alloc(),
            free: self.free(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{
        ptr::null_mut,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use crate::{decoder_builder, encoder_builder};

    use super::*;

    pub struct NoManager {}

    impl MemoryManager for NoManager {
        fn alloc(&self) -> AllocFn {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C" fn alloc(_opaque: *mut c_void, _size: usize) -> *mut c_void {
                null_mut()
            }

            alloc
        }

        fn free(&self) -> FreeFn {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C" fn free(_opaque: *mut c_void, _address: *mut c_void) {
                debug_assert!(false, "Should not be called");
            }

            free
        }
    }

    /// Example implementation of [`MemoryManager`] of a fixed size allocator
    pub struct BumpManager<const N: usize> {
        arena: Box<[u8; N]>,
        footer: AtomicUsize,
    }

    impl<const N: usize> Default for BumpManager<N> {
        fn default() -> Self {
            Self {
                arena: Box::new([0_u8; N]),
                footer: AtomicUsize::new(0),
            }
        }
    }

    impl<const N: usize> MemoryManager for BumpManager<N> {
        fn alloc(&self) -> AllocFn {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C" fn alloc<const N: usize>(
                opaque: *mut c_void,
                size: usize,
            ) -> *mut c_void {
                let mm = &mut *opaque.cast::<BumpManager<{ N }>>();

                let footer = mm.footer.load(Ordering::Acquire);
                let mut new = footer + size;

                loop {
                    if new > N {
                        break null_mut();
                    } else if let Err(s) = mm.footer.compare_exchange_weak(
                        footer,
                        new,
                        Ordering::AcqRel,
                        Ordering::Relaxed,
                    ) {
                        new = s + size;
                    } else {
                        let addr = mm.arena.get_unchecked_mut(footer);
                        #[allow(clippy::ref_as_ptr)]
                        break (addr as *mut u8).cast();
                    }
                }
            }

            alloc::<N>
        }

        fn free(&self) -> FreeFn {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C" fn free(_opaque: *mut c_void, _address: *mut c_void) {}

            free
        }
    }

    #[test]
    fn test_mm() {
        let mm = NoManager {};
        assert!(decoder_builder().memory_manager(&mm).build().is_err());
        assert!(encoder_builder().memory_manager(&mm).build().is_err());

        let mm = BumpManager::<{ 1024 * 10 }>::default();
        assert!(decoder_builder().memory_manager(&mm).build().is_ok());
        assert!(encoder_builder().memory_manager(&mm).build().is_ok());
    }
}
