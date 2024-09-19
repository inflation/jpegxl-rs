/*
 * This file is part of jpegxl-rs.
 *
 * jpegxl-rs is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * jpegxl-rs is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Memory manager interface

use jpegxl_sys::common::memory_manager::{JpegxlAllocFunc, JpegxlFreeFunc, JxlMemoryManager};

/// General trait for a memory manager

#[allow(clippy::module_name_repetitions)]
pub trait MemoryManager {
    /// Return a custom allocating function
    fn alloc(&self) -> JpegxlAllocFunc;
    /// Return a custom deallocating function
    fn free(&self) -> JpegxlFreeFunc;

    /// Helper conversion function for C API
    #[must_use]
    fn manager(&self) -> JxlMemoryManager {
        JxlMemoryManager {
            opaque: (self as *const Self).cast_mut().cast(),
            alloc: Some(self.alloc()),
            free: Some(self.free()),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{
        ffi::c_void,
        ptr::null_mut,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use testresult::TestResult;

    use crate::{decoder_builder, encoder_builder};

    use super::*;
    /// Example implementation of [`MemoryManager`] of a fixed size allocator
    pub struct BumpManager {
        arena: Vec<u8>,
        footer: AtomicUsize,
    }

    impl BumpManager {
        pub(crate) fn new(n: usize) -> Self {
            Self {
                arena: vec![0; n],
                footer: AtomicUsize::new(0),
            }
        }
    }

    impl MemoryManager for BumpManager {
        fn alloc(&self) -> JpegxlAllocFunc {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C-unwind" fn alloc(opaque: *mut c_void, size: usize) -> *mut c_void {
                let mm = &mut *opaque.cast::<BumpManager>();

                let footer = mm.footer.load(Ordering::Acquire);
                let mut new = footer + size;

                loop {
                    if new > mm.arena.len() {
                        println!("Out of memory");
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
                        break (addr as *mut u8).cast();
                    }
                }
            }

            alloc
        }

        fn free(&self) -> JpegxlFreeFunc {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C-unwind" fn free(_opaque: *mut c_void, _address: *mut c_void) {}

            free
        }
    }
    pub struct PanicManager {}

    impl MemoryManager for PanicManager {
        fn alloc(&self) -> JpegxlAllocFunc {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C-unwind" fn alloc(_opaque: *mut c_void, _size: usize) -> *mut c_void {
                panic!("Stack unwind test")
            }

            alloc
        }

        fn free(&self) -> JpegxlFreeFunc {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C-unwind" fn free(_opaque: *mut c_void, _address: *mut c_void) {
                debug_assert!(false, "Should not be called");
            }

            free
        }
    }

    #[test]
    fn test_mm() -> TestResult {
        let mm = BumpManager::new(1024 * 1024 * 50);
        let dec = decoder_builder().memory_manager(&mm).build()?;
        let (meta, img) = dec.decode_with::<u8>(crate::tests::SAMPLE_JXL)?;

        let mut enc = encoder_builder().memory_manager(&mm).build()?;
        let _ = enc.encode::<u8, u8>(&img, meta.width, meta.height)?;

        Ok(())
    }

    #[test]
    #[should_panic = "Stack unwind test"]
    fn test_unwind() {
        let mm = PanicManager {};
        let _ = decoder_builder().memory_manager(&mm).build().unwrap();
    }
}
