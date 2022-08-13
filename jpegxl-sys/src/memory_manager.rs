use std::ffi::c_void;

pub type JpegxlAllocFunc = unsafe extern "C" fn(opaque: *mut c_void, size: usize) -> *mut c_void;
pub type JpegxlFreeFunc = unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void);

#[repr(C)]
pub struct JxlMemoryManager {
    pub opaque: *mut c_void,
    pub alloc: JpegxlAllocFunc,
    pub free: JpegxlFreeFunc,
}
