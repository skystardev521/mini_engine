use crate::epoll::Epoll;
use crate::epoll::EpollEvent;
use libc;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use utils::native;

const LISTEN_ID: u64 = 0;
const MIN_EVENT: u16 = 8;

pub struct TcpListen<'a, 'b> {
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
    pub fn run(&mut self, timeout: i32) -> Result<(), String> {
        match self.epoll.wait(timeout, &mut self.vec_epoll_event) {
            Ok(0) => return Ok(()),
            Ok(num) => {
                for n in 0..num as usize {
                    let event = self.vec_epoll_event[n];
                    if event.u64 == LISTEN_ID {
                        match self.loop_accept() {
                            Ok(()) => continue,
                            Err(err) => return Err(err),
                        }
                    }

                    if (event.events & libc::EPOLLIN as u32) != 0 {
                        self.epoll_event.read(event.u64);
                    }
                    if (event.events & libc::EPOLLOUT as u32) != 0 {
                        self.epoll_event.write(event.u64);
                    }
                    if (event.events & libc::EPOLLERR as u32) != 0 {
                        self.epoll_event.error(event.u64, native::c_strerr());
                    }
                    //if event.events & libc::EPOLLHUP {}  | libc::EPOLLHUP
                }
                return Ok(());
            }
            Err(err) => return Err(err),
        }
    }

    #[inline]
    fn loop_accept(&mut self) -> Result<(), String> {
        loop {
            match self.listen.accept() {
                Ok((socket, addr)) => {
                    self.epoll_event.accept(socket, addr);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(err) => return Err(format!("listen.accept() error:{}", err)),
            }
        }
        Ok(())
    }
}
