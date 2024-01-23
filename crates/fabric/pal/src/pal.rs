#![allow(non_camel_case_types, non_snake_case, dead_code)]
use std::ffi::c_void;
use std::mem::size_of;

use libc::{__errno_location, malloc};
use windows::core::imp::LOAD_LIBRARY_FLAGS;
use windows::Win32::Foundation::{
    ERROR_BAD_ARGUMENTS, ERROR_NOT_ENOUGH_MEMORY, E_POINTER, STATUS_HEAP_CORRUPTION,
};
use windows::{
    core::{HRESULT, PCSTR, PWSTR},
    Win32::Foundation::{HANDLE, HMODULE},
};

static DUMMY_HEAP: isize = 0x01020304;

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn GetLastError() -> u32 {
    let pe = __errno_location();
    if !pe.is_null() {
        *pe as u32
    } else {
        0
    }
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn SetLastError(dwerrcode: u32) {
    let pe = __errno_location();
    if !pe.is_null() {
        *pe = dwerrcode as i32
    }
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn GetProcessHeap() -> isize {
    DUMMY_HEAP
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn HeapAlloc(heap: isize, _flags: u32, len: usize) -> *mut c_void {
    if heap != DUMMY_HEAP {
        return std::ptr::null_mut();
    }

    if len == 0 {
        SetLastError(ERROR_BAD_ARGUMENTS.0);
        return std::ptr::null_mut();
    }

    // use vec to allocate contiguous memory. The first 2 bytes is used to store buff len.
    let prefix_len = size_of::<usize>();
    let mut vec = Vec::<u8>::with_capacity(len + prefix_len);
    // write len to the prefix
    let prefix_ptr = vec.as_mut_ptr() as *mut usize;
    *prefix_ptr = len;

    // return the vec buf at the data segment
    let ptr_out = vec.as_mut_ptr().offset(prefix_len as isize);
    std::mem::forget(vec);
    ptr_out as *mut c_void
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn HeapFree(heap: isize, _flags: u32, ptr: *const c_void) -> i32 {
    if heap != DUMMY_HEAP {
        SetLastError(STATUS_HEAP_CORRUPTION.0 as u32);
        return 0; // fail to free
    }

    // do not support free null ptr.
    if ptr.is_null() {
        SetLastError(E_POINTER.0 as u32);
        return 0;
    }

    // get vec cap. prefix is the out buffer size only.
    let prefix_len = size_of::<usize>();
    let prefix_ptr = ptr.offset(-(prefix_len as isize)) as *const usize;
    let cap = *prefix_ptr + prefix_len;

    let vec = Vec::from_raw_parts(prefix_ptr as *mut u8, 0, cap);
    std::mem::drop(vec);
    1 // success
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn GetErrorInfo(_reserved: u32, _info: *mut *mut c_void) -> HRESULT {
    HRESULT(0)
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn SetErrorInfo(_reserved: u32, _info: *const c_void) -> HRESULT {
    HRESULT(0)
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn LoadLibraryA(_name: PCSTR) -> isize {
    0
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn LoadLibraryExA(
    _lplibfilename: PCSTR,
    _hfile: HANDLE,
    _dwflags: LOAD_LIBRARY_FLAGS,
) -> HMODULE {
    windows::Win32::Foundation::HMODULE(0)
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn FreeLibrary(_library: isize) -> i32 {
    0
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn GetProcAddress(_library: isize, _name: PCSTR) -> *const c_void {
    std::ptr::null()
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn SysFreeString(_bstr: *const u16) {}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn SysStringLen(_bstr: *const u16) -> u32 {
    0
}

/// # Safety
///
/// safe
#[no_mangle]
pub unsafe extern "system" fn FormatMessageW(
    _flags: u32,
    _source: *const c_void,
    _code: u32,
    _lang: u32,
    _buffer: PWSTR,
    _len: u32,
    _args: *const *const i8,
) -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::pal::HeapFree;

    use super::{GetProcessHeap, HeapAlloc};

    #[test]
    fn test_memory_alloc() {
        let p = unsafe { HeapAlloc(GetProcessHeap(), 0, size_of::<usize>()) };
        assert!(!p.is_null());

        let p_size = p as *mut usize;
        unsafe { *p_size = 99 };

        let ok = unsafe { HeapFree(GetProcessHeap(), 0, p) };
        assert_ne!(ok, 0);
    }
}
