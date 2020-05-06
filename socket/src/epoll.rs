use libc;
use std::os::unix::io::RawFd;

use crate::utils;
use utils::ffi_ext;

#[derive(Debug)]
pub struct Epoll {
    epoll_fd: libc::c_int,
}

impl Drop for Epoll {
    fn drop(&mut self) {
        if self.epoll_fd != -1 {
            unsafe { libc::close(self.epoll_fd) };
        }
    }
}

impl Epoll {
    pub fn new() -> Result<Self, String> {
        let mut epoll = Epoll { epoll_fd: -1 };
        unsafe {
            let fd = libc::epoll_create1(0);
            if fd != -1 {
                epoll.epoll_fd = fd;
                return Ok(epoll);
            } else {
                return Err(ffi_ext::());
            }
        }
    }

    #[inline]
    pub fn ctl_add_fd(&self, id: u64, fd: RawFd, ev: i32) -> Result<(), String> {
        let mut event = libc::epoll_event {
            u64: (id as libc::c_ulonglong),
            events: (libc::EPOLLET | ev) as u32,
        };
        unsafe {
            let ret = libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(utils::c_err_string());
        }
    }
    #[inline]
    pub fn ctl_mod_fd(&self, id: u64, fd: RawFd, ev: i32) -> Result<(), String> {
        let mut event = libc::epoll_event {
            u64: (id as libc::c_ulonglong),
            events: (libc::EPOLLET | ev) as u32,
        };
        unsafe {
            let ret = libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(utils::c_err_string());
        }
    }
    #[inline]
    pub fn ctl_del_fd(&self, id: u64, fd: RawFd) -> Result<(), String> {
        let mut event = libc::epoll_event {
            events: 0,
            u64: (id as libc::c_ulonglong),
        };

        unsafe {
            let ret = libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_DEL, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(utils::c_err_string());
        }
    }
    #[inline]
    pub fn wait(&mut self, events: &mut Vec<libc::epoll_event>, timeout: i32) -> i32 {
        unsafe { libc::epoll_wait(self.epoll_fd, &mut events[0], events.len() as i32, timeout) }
    }
}
