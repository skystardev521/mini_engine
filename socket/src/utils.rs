use libc;
use std::ffi::CStr;
use std::io::ErrorKind;
use std::mem;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

//pub static NEXT_ID: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));

//#[inline]
//pub fn get_au64_id() -> u64 {
 //   NEXT_ID.fetch_add(1, Ordering::SeqCst)
//}

#[inline]
///total(v1 + v2 + v3)
///value(1,2,4,8,16,32,64,128)
pub fn bit_match(total: u8, val: u8) -> bool {
    (total & val) == val
}

#[inline]
pub fn c_err_string() -> String {
    unsafe {
        let str_err = libc::strerror(*libc::__errno_location());
        return CStr::from_ptr(str_err).to_string_lossy().into_owned();
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
            let st = c_err_string();
            return Err(st);
        }
    }
}
pub fn error_kind_string(error: ErrorKind) -> String {
    let str = match error {
        ErrorKind::NotFound => "entity not found",
        ErrorKind::PermissionDenied => "permission denied",
        ErrorKind::ConnectionRefused => "connection refused",
        ErrorKind::ConnectionReset => "connection reset",
        ErrorKind::ConnectionAborted => "connection aborted",
        ErrorKind::NotConnected => "not connected",
        ErrorKind::AddrInUse => "address in use",
        ErrorKind::AddrNotAvailable => "address not available",
        ErrorKind::BrokenPipe => "broken pipe",
        ErrorKind::AlreadyExists => "entity already exists",
        ErrorKind::WouldBlock => "operation would block",
        ErrorKind::InvalidInput => "invalid input parameter",
        ErrorKind::InvalidData => "invalid data",
        ErrorKind::TimedOut => "timed out",
        ErrorKind::WriteZero => "write zero",
        ErrorKind::Interrupted => "operation interrupted",
        ErrorKind::Other => "other os error",
        ErrorKind::UnexpectedEof => "unexpected end of file",
        _ => "unknow error",
    };
    String::from(str)
}
