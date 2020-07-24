use libc;

use std::io::Error;
use std::mem;
use std::os::unix::io::RawFd;

/*
#[inline]
///total(v1 + v2 + v3)
///value(1,2,4,8,16,32,64,128)
pub fn bit_match(total: u8, val: u8) -> bool {
    (total & val) == val
}
*/

/// os_socket::setsockopt(raw_fd,libc::SOL_SOCKET,libc::SO_SNDBUF, 8192)
#[inline]
pub fn setsockopt<T>(fd: RawFd, opt: libc::c_int, key: libc::c_int, val: T) -> Result<(), String> {
    unsafe {
        let ret = libc::setsockopt(
            fd,
            opt,
            key,
            &val as *const T as *const libc::c_void,
            mem::size_of::<T>() as libc::socklen_t,
        );
        if ret == 0 {
            return Ok(());
        } else {
            return Err(Error::last_os_error().to_string());
        }
    }
}

//os_socket::getsockopt::<i32>(raw_fd, libc::SOL_SOCKET, libc::SO_SNDBUF)
#[inline]
pub fn getsockopt<T: Copy>(fd: RawFd, opt: libc::c_int, key: libc::c_int) -> Result<T, String> {
    unsafe {
        let mut val: T = mem::zeroed();
        let ret = libc::getsockopt(
            fd,
            opt,
            key,
            &mut val as *mut T as *mut libc::c_void,
            &mut (mem::size_of::<T>() as libc::socklen_t),
        );
        if ret == 0 {
            return Ok(val);
        } else {
            return Err(Error::last_os_error().to_string());
        }
    }
}
