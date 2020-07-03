use libc;
use std::os::unix::io::RawFd;
use utils::native;
use std::io::Error;

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
    pub fn new() -> Result<Self, String> {
        let mut epoll = Epoll { fd: -1 };
        unsafe {
            let fd = libc::epoll_create1(0);
            if fd != -1 {
                epoll.fd = fd;
                return Ok(epoll);
            } else {
                return Err(Error::last_os_error().to_string());
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
            let ret = libc::epoll_ctl(self.fd, libc::EPOLL_CTL_ADD, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(Error::last_os_error().to_string());
        }
    }
    #[inline]
    pub fn ctl_mod_fd(&self, id: u64, fd: RawFd, ev: i32) -> Result<(), String> {
        let mut event = libc::epoll_event {
            u64: (id as libc::c_ulonglong),
            events: (libc::EPOLLET | ev) as u32,
        };
        unsafe {
            let ret = libc::epoll_ctl(self.fd, libc::EPOLL_CTL_MOD, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(Error::last_os_error().to_string());
        }
    }
    #[inline]
    pub fn ctl_del_fd(&self, id: u64, fd: RawFd) -> Result<(), String> {
        let mut event = libc::epoll_event {
            events: 0,
            u64: (id as libc::c_ulonglong),
        };

        unsafe {
            let ret = libc::epoll_ctl(self.fd, libc::EPOLL_CTL_DEL, fd, &mut event);
            if ret != -1 {
                return Ok(());
            }
            return Err(Error::last_os_error().to_string());
        }
    }
    #[inline]
    pub fn wait(&self, timeout: i32, events: &mut Vec<libc::epoll_event>) -> Result<u32, String> {
        unsafe {
            let ret = libc::epoll_wait(self.fd, &mut events[0], events.len() as i32, timeout);
            if ret > -1 {
                return Ok(ret as u32);
            }
            if libc::EINTR == *libc::__errno_location() {
                return Ok(0);
            }
            return Err(Error::last_os_error().to_string());
        }
    }
}
