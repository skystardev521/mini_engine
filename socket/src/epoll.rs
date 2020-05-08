use libc;
use std::os::unix::io::RawFd;

use utils::ffi_ext;

#[derive(Debug)]
pub struct Epoll {
    fd: libc::c_int,
}

impl Drop for Epoll {
    fn drop(&mut self) {
        if self.fd != -1 {
            unsafe { libc::close(self.fd) };
        }
    }
}

impl Epoll {
    pub fn new() -> Result<Self, &'static str> {
        let mut epoll = Epoll { fd: -1 };
        unsafe {
            let fd = libc::epoll_create1(0);
            if fd != -1 {
                epoll.fd = fd;
                return Ok(epoll);
            } else {
                return Err(ffi_ext::strerror());
            }
        }
    }

    #[inline]
    pub fn ctl_add_fd(&self, id: u64, fd: RawFd, ev: i32) -> Result<(), &'static str> {
        let mut event = libc::epoll_event {
            u64: (id as libc::c_ulonglong),
            events: (libc::EPOLLET | ev) as u32,
        };
        unsafe {
            let ret = libc::epoll_ctl(self.fd, libc::EPOLL_CTL_ADD, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(ffi_ext::strerror());
        }
    }
    #[inline]
    pub fn ctl_mod_fd(&self, id: u64, fd: RawFd, ev: i32) -> Result<(), &'static str> {
        let mut event = libc::epoll_event {
            u64: (id as libc::c_ulonglong),
            events: (libc::EPOLLET | ev) as u32,
        };
        unsafe {
            let ret = libc::epoll_ctl(self.fd, libc::EPOLL_CTL_MOD, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(ffi_ext::strerror());
        }
    }
    #[inline]
    pub fn ctl_del_fd(&self, id: u64, fd: RawFd) -> Result<(), &'static str> {
        let mut event = libc::epoll_event {
            events: 0,
            u64: (id as libc::c_ulonglong),
        };

        unsafe {
            let ret = libc::epoll_ctl(self.fd, libc::EPOLL_CTL_DEL, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(ffi_ext::strerror());
        }
    }
    #[inline]
    pub fn wait(
        &mut self,
        events: &mut Vec<libc::epoll_event>,
        timeout: i32,
    ) -> Result<u32, &'static str> {
        unsafe {
            let ret = libc::epoll_wait(self.fd, &mut events[0], events.len() as i32, timeout);
            if ret > -1 {
                return Ok(ret as u32);
            }
            return Err(ffi_ext::strerror());
        }
    }
}
