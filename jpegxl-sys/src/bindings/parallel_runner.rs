use std::ffi::c_void;

pub type JxlParallelRetCode = i32;

pub type JxlParallelRunInit =
    unsafe extern "C" fn(jpegxl_opaque: *mut c_void, num_threads: usize) -> JxlParallelRetCode;

pub type JxlParallelRunFunction =
    unsafe extern "C" fn(jpegxl_opaque: *mut c_void, value: u32, thread_id: usize);

pub type JxlParallelRunner = unsafe extern "C" fn(
    runner_opaque: *mut c_void,
    jpegxl_opaque: *mut c_void,
    init: JxlParallelRunInit,
    func: JxlParallelRunFunction,
    start_range: u32,
    end_range: u32,
) -> JxlParallelRetCode;
