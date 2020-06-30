use libc;
use std::ffi::CStr;
use std::mem;
use std::os::unix::io::RawFd;

#[inline]
///total(v1 + v2 + v3)
///value(1,2,4,8,16,32,64,128)
pub fn bit_match(total: u8, val: u8) -> bool {
    (total & val) == val
}

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
