use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use utils::native;

pub struct TcpBind {
    listen: TcpListener,
}

impl TcpBind {
    pub fn new(socket_addr: &String) -> Result<Self, String> {
        let listen = match TcpListener::bind(socket_addr) {
            Ok(listen) => listen,
            Err(err) => return Err(format!("{}", err)),
        };

        match listen.set_nonblocking(true) {
            Ok(()) => (),
            Err(err) => return Err(format!("{}", err)),
        }

        match native::setsockopt(listen.as_raw_fd(), libc::SO_REUSEADDR, 1) {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        Ok(TcpBind { listen })
    }

    #[inline]
    pub fn get_listen(&self) -> &TcpListener {
        &self.listen
    }
}
/*
use crate::epoll::Epoll;
use crate::epoll::EpollEvent;
use libc;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use utils::native;

const LISTEN_ID: u64 = 0;
const MIN_EVENT: u16 = 8;

pub struct TcpBind<'a, 'b> {
    epoll: &'a Epoll,
    listen: TcpListener,
    epoll_event: &'b mut dyn EpollEvent,
    vec_epoll_event: Vec<libc::epoll_event>,
}

impl<'a, 'b> TcpListen<'a, 'b> {
    pub fn new(
        addr: &String,
        maxevents: u16,
        epoll: &'a Epoll,
        epoll_event: &'b mut dyn EpollEvent,
    ) -> Result<Self, String> {
        let mut max_events = maxevents;
        if max_events < MIN_EVENT {
            max_events = MIN_EVENT
        }
        let listen = match TcpListener::bind(addr) {
            Ok(listen) => listen,
            Err(err) => return Err(format!("{}", err)),
        };

        match listen.set_nonblocking(true) {
            Ok(()) => (),
            Err(err) => return Err(format!("{}", err)),
        }

        match native::setsockopt(listen.as_raw_fd(), libc::SO_REUSEADDR, 1) {
            Ok(()) => (),
            Err(err) => return Err(err),
        }

        match epoll.ctl_add_fd(LISTEN_ID, listen.as_raw_fd(), libc::EPOLLIN) {
            Ok(()) => (),
            Err(err) => return Err(err),
        }

        Ok(TcpListen {
            epoll: epoll,
            listen: listen,
            epoll_event: epoll_event,
            vec_epoll_event: vec![libc::epoll_event { events: 0, u64: 0 }; max_events as usize],
        })
    }

    /// timeout ms
    pub fn wait(&mut self, timeout: i32) -> Result<bool, String> {
        match self.epoll.wait(timeout, &mut self.vec_epoll_event) {
            Ok(0) => return Ok(true),
            Ok(num) => {
                for n in 0..num as usize {
                    let event = self.vec_epoll_event[n];
                    if event.u64 == LISTEN_ID {
                        match self.loop_accept() {
                            Ok(_) => continue,
                            Err(err) => return Err(err),
                        }
                    }

                    if (event.events & libc::EPOLLIN as u32) != 0 {
                        if self.epoll_event.read(event.u64) == false {
                            return Ok(false);
                        }
                    }
                    if (event.events & libc::EPOLLOUT as u32) != 0 {
                        if self.epoll_event.write(event.u64) == false {
                            return Ok(false);
                        }
                    }
                    if (event.events & libc::EPOLLERR as u32) != 0 {
                        if self.epoll_event.error(event.u64, native::c_strerr()) == false {
                            return Ok(false);
                        }
                    }
                    //if event.events & libc::EPOLLHUP {}  | libc::EPOLLHUP
                }
                return Ok(true);
            }
            Err(err) => return Err(err),
        }
    }

    fn loop_accept(&mut self) -> Result<bool, String> {
        loop {
            match self.listen.accept() {
                Ok((stream, _)) => {
                    if self.epoll_event.accept(stream) == false {
                        return Ok(false);
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(err) => return Err(format!("listen.accept() error:{}", err)),
            }
        }
        Ok(true)
    }
}
*/
