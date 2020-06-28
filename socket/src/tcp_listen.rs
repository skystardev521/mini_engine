use crate::epoll::Epoll;
use crate::epoll::EpollEvent;
use libc;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use utils::capi;

const LISTEN_ID: u64 = 0;
const MIN_EVENT: u16 = 8;

/*
pub struct TcpListen<'a> {
    epoll: &'a Epoll,
    listen: TcpListener,
    epevent: &'a mut EpEvent<'a>,
    epevents: Vec<libc::epoll_event>,
}
*/

pub struct TcpListen<'a> {
    epoll: &'a Epoll,
    listen: TcpListener,
    epevent: &'a mut dyn EpollEvent,
    epevents: Vec<libc::epoll_event>,
}

impl<'a> TcpListen<'a> {
    pub fn new(
        addr: &String,
        maxevents: u16,
        epoll: &'a Epoll,
        epevent: &'a mut dyn EpollEvent,
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

        match capi::setsockopt(listen.as_raw_fd(), libc::SO_REUSEADDR, 1) {
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
            epevent: epevent,
            epevents: vec![libc::epoll_event { events: 0, u64: 0 }; max_events as usize],
        })
    }

    /// timeout ms
    pub fn run(&mut self, timeout: i32) -> Result<(), String> {
        match self.epoll.wait(timeout, &mut self.epevents) {
            Ok(0) => return Ok(()),
            Ok(num) => {
                for n in 0..num as usize {
                    let event = self.epevents[n];
                    if event.u64 == LISTEN_ID {
                        match self.loop_accept() {
                            Ok(()) => continue,
                            Err(err) => return Err(err),
                        }
                    }

                    if (event.events & libc::EPOLLIN as u32) != 0 {
                        self.epevent.read(event.u64);
                    }
                    if (event.events & libc::EPOLLOUT as u32) != 0 {
                        self.epevent.write(event.u64);
                    }
                    if (event.events & libc::EPOLLERR as u32) != 0 {
                        self.epevent.error(event.u64, capi::c_strerr());
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
                    self.epevent.accept(socket, addr);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(err) => return Err(format!("listen.accept() result:{}", err)),
            }
        }
        Ok(())
    }
}
