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

pub(crate) type MemoryManagerRef = jpegxl_sys::JxlMemoryManager;

/// Allocator function type
pub type AllocFn = unsafe extern "C" fn(opaque: *mut c_void, size: usize) -> *mut c_void;
/// Deallocator function type
pub type FreeFn = unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void);

/// General trait for a memory manager

pub trait JxlMemoryManager {
    /// Return a custom allocator function. Can be None for using default one
    fn alloc(&self) -> AllocFn;
    /// Return a custom deallocator function. Can be None for using default one
    fn free(&self) -> FreeFn;

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
    use std::ptr::null_mut;

    use crate::{decoder_builder, encoder_builder, DecodeError, EncodeError, ThreadsRunner};

    use super::*;

    pub struct NoManager {}

    impl JxlMemoryManager for NoManager {
        fn alloc(&self) -> AllocFn {
            unsafe extern "C" fn alloc(_a: *mut c_void, _b: usize) -> *mut c_void {
                null_mut()
            }

            alloc
        }

        fn free(&self) -> FreeFn {
            unsafe extern "C" fn free(_opaque: *mut c_void, _address: *mut c_void) {}

            free
        }
    }

    /// Example implementation of [`JxlMemoryManager`] of a fixed size allocator
    pub struct BumpManager<const N: usize> {
        arena: Box<[u8; N]>,
        footer: usize,
    }

    impl<const N: usize> Default for BumpManager<N> {
        fn default() -> Self {
            Self {
                arena: Box::new([0_u8; N]),
                footer: 0,
            }
        }
    }

    impl<const N: usize> JxlMemoryManager for BumpManager<N> {
        fn alloc(&self) -> AllocFn {
            unsafe extern "C" fn alloc<const N: usize>(
                opaque: *mut c_void,
                size: usize,
            ) -> *mut c_void {
                let mm = &mut *opaque.cast::<BumpManager<{ N }>>();
                if mm.footer + size > N {
                    null_mut()
                } else {
                    let addr = mm.arena.get_unchecked_mut(mm.footer);
                    mm.footer += size;
                    (addr as *mut u8).cast()
                }
            }

            alloc::<N>
        }

        fn free(&self) -> FreeFn {
            unsafe extern "C" fn free(_opaque: *mut c_void, _address: *mut c_void) {}

            free
        }
    }

    #[test]
    fn test_no_manager() {
        let mm = NoManager {};

        let decoder = decoder_builder().build_with(&mm);
        assert!(matches!(decoder, Err(DecodeError::CannotCreateDecoder)));

        let encoder = encoder_builder().build_with(&mm);
        assert!(matches!(encoder, Err(EncodeError::CannotCreateEncoder)));
    }

    #[test]
    fn test_bump_manager() -> Result<(), DecodeError> {
        let sample = std::fs::read("samples/sample.jxl")?;
        let mm = BumpManager::<{ 1024 * 5 }>::default();
        let parallel_runner = ThreadsRunner::new(Some(&mm.to_manager()), None)
            .expect("Failed to create ThreadsRunner");
        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build_with(&mm)?;

        decoder.decode::<u8>(&sample)?;

        Ok(())
    }
}
