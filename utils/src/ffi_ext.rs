
use libc;
use std::ffi::CStr;

#[inline]
pub fn strerror() -> String {
    unsafe {
        let str_err = libc::strerror(*libc::__errno_location());
        return CStr::from_ptr(str_err).to_string_lossy().into_owned();
    }
}