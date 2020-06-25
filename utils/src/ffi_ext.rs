use libc;
use std::ffi::CStr;
use std::mem;
use std::os::unix::io::RawFd;

/*
#[inline]
pub fn strerror() -> &'static str {
    unsafe {
        let str_err = libc::strerror(*libc::__errno_location());
        if str_err.is_null() {
            return "";
        }
        match CStr::from_ptr(str_err).to_str() {
            Ok(result) => result,
            Err(_e) => "libc::strerror to_str error",
        }
    }
}
*/

#[inline]
pub fn c_strerr() -> String {
    unsafe {
        let cstr = libc::strerror(*libc::__errno_location());
        CStr::from_ptr(cstr).to_string_lossy().to_string()
    }
}

pub fn setsockopt<T>(fd: RawFd, opt_key: libc::c_int, opt_val: T) -> Result<(), String> {
    unsafe {
        let ret = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            opt_key,
            &opt_val as *const T as *const libc::c_void,
            mem::size_of::<T>() as libc::socklen_t,
        );
        if ret == 0 {
            return Ok(());
        } else {
            return Err(c_strerr());
        }
    }
}
