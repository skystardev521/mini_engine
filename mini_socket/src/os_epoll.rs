use libc;
use std::io::Error;
use std::os::unix::io::RawFd;

const EPOLL_EVENTS: i32 = (libc::EPOLLET | libc::EPOLLERR) as i32;

#[derive(Debug)]
pub struct OSEpoll {
    fd: libc::c_int,
}

impl Drop for OSEpoll {
    fn drop(&mut self) {
        if self.fd != -1 {
            unsafe { libc::close(self.fd) };
        }
    }
}

impl OSEpoll {
    pub fn new() -> Result<Self, String> {
        let mut os_epoll = OSEpoll { fd: -1 };
        unsafe {
            let fd = libc::epoll_create1(0);
            if fd != -1 {
                os_epoll.fd = fd;
                return Ok(os_epoll);
            } else {
                return Err(Error::last_os_error().to_string());
            }
        }
    }

    #[inline]
    pub fn ctl_add_fd(&self, id: u64, fd: RawFd, ev: i32) -> Result<(), String> {
        let mut event = libc::epoll_event {
            u64: id,//as libc::c_ulonglong),
            events: (EPOLL_EVENTS | ev) as u32,
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
            u64: id,//as libc::c_ulonglong),
            events: (EPOLL_EVENTS | ev) as u32,
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
            u64: id,//as libc::c_ulonglong),
            events: 0,
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
            //println!("ret:{} epoll_event:{}", ret, mini_utils::time::timestamp());
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
