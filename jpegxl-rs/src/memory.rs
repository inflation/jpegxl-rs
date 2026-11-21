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
///
/// # Safety
/// The callbacks and context returned by [`Self::manager`] must stay valid
/// while the manager is borrowed, including calls from native worker threads.
/// Allocation must return null on failure or a suitably aligned, exclusive
/// allocation of at least the requested size. Free must accept allocations
/// from the same manager and null. Concurrent calls must be synchronized.
#[allow(clippy::module_name_repetitions)]
pub unsafe trait MemoryManager {
    /// Return a custom allocating function
    fn alloc(&self) -> JpegxlAllocFunc;
    /// Return a custom deallocating function
    fn free(&self) -> JpegxlFreeFunc;

    /// Helper conversion function for C API
    #[must_use]
    fn manager(&self) -> JxlMemoryManager {
        JxlMemoryManager {
            opaque: std::ptr::from_ref(self).cast_mut().cast(),
            alloc: Some(self.alloc()),
            free: Some(self.free()),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::cell::UnsafeCell;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::Mutex;

    use testresult::TestResult;

    use super::*;
    use crate::{decoder_builder, encoder_builder};
    /// Example implementation of [`MemoryManager`] of a fixed size allocator
    pub struct BumpManager {
        arena: Box<[UnsafeCell<u128>]>,
        next_word: Mutex<usize>,
    }

    impl BumpManager {
        pub(crate) fn new(n: usize) -> Self {
            Self {
                arena: (0..n / size_of::<u128>())
                    .map(|_| UnsafeCell::new(0))
                    .collect(),
                next_word: Mutex::new(0),
            }
        }
    }

    // SAFETY: locked reservations give each caller disjoint aligned storage until drop.
    unsafe impl MemoryManager for BumpManager {
        fn alloc(&self) -> JpegxlAllocFunc {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C-unwind" fn alloc(opaque: *mut c_void, size: usize) -> *mut c_void {
                // SAFETY: manager() supplies the borrowed BumpManager as context.
                let mm = unsafe { &*opaque.cast::<BumpManager>() };
                let words = size.max(1).div_ceil(size_of::<u128>());
                let mut next = mm.next_word.lock().unwrap();
                let offset = *next;
                if words > mm.arena.len() - offset {
                    return null_mut();
                }
                *next += words;
                // SAFETY: the reservation covers at least one cell in the live arena.
                unsafe { UnsafeCell::raw_get(mm.arena.as_ptr().add(offset)).cast() }
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

    // SAFETY: the allocation callback unwinds before handing out any storage.
    unsafe impl MemoryManager for PanicManager {
        fn alloc(&self) -> JpegxlAllocFunc {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C-unwind" fn alloc(_opaque: *mut c_void, _size: usize) -> *mut c_void {
                panic!("Stack unwind test")
            }

            alloc
        }

        fn free(&self) -> JpegxlFreeFunc {
            #[cfg_attr(coverage_nightly, coverage(off))]
            unsafe extern "C-unwind" fn free(_opaque: *mut c_void, address: *mut c_void) {
                debug_assert!(address.is_null(), "no allocations were returned");
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
        let _ = enc.encode(&img, meta.width, meta.height)?;

        Ok(())
    }

    #[test]
    fn out_of_memory_fails_creation() {
        use crate::{DecodeError, EncodeError, ThreadsRunner};

        let mm = BumpManager::new(0);
        assert!(matches!(
            decoder_builder().memory_manager(&mm).build(),
            Err(DecodeError::CannotCreateDecoder)
        ));
        assert!(matches!(
            encoder_builder().memory_manager(&mm).build(),
            Err(EncodeError::CannotCreateEncoder)
        ));
        assert!(ThreadsRunner::new(Some(&mm), None).is_none());
    }

    #[test]
    #[should_panic = "Stack unwind test"]
    fn test_unwind() {
        let mm = PanicManager {};
        let _ = decoder_builder().memory_manager(&mm).build().unwrap();
    }
}
