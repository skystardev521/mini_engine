use crate::epoll::Epoll;
use crate::tcp_event::TcpEvent;
use libc;
use std::io::{Error, ErrorKind};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use utils::ffi_ext;

pub struct TcpListen<'a> {
    token: u64,
    epoll: Epoll,
    tcp_event: TcpEvent<'a>,
    tcp_listen: TcpListener,
    events: Vec<libc::epoll_event>,
}

impl<'a> TcpListen<'a> {
    pub fn new(
        token: u64,
        events: u16,
        socekt_addr: &String,
        tcp_event: TcpEvent<'a>,
    ) -> Result<Self, Error> {
        let mut max_event = events;
        if max_event < 16 {
            max_event = 16
        }

        let epoll = match Epoll::new() {
            Ok(epoll) => epoll,
            Err(err) => return Err(Error::new(ErrorKind::Other, err)),
        };

        let tcp_listen = match TcpListener::bind(socekt_addr) {
            Ok(tcp_listen) => tcp_listen,
            Err(err) => return Err(err),
        };

        match &tcp_listen.set_nonblocking(true) {
            Ok(()) => (),
            Err(_err) => return Err(Error::new(ErrorKind::Other, "cannot set non-blocking")),
        }

        match ffi_ext::setsockopt(tcp_listen.as_raw_fd(), libc::SO_REUSEADDR, 1) {
            Ok(()) => (),
            Err(err) => return Err(Error::new(ErrorKind::Other, err)),
        }

        match epoll.ctl_add_fd(token as u64, tcp_listen.as_raw_fd(), libc::EPOLLIN) {
            Ok(()) => (),
            Err(err) => return Err(Error::new(ErrorKind::Other, err)),
        }

        Ok(TcpListen {
            token: token,
            epoll: epoll,
            tcp_event: tcp_event,
            tcp_listen: tcp_listen,
            events: vec![libc::epoll_event { events: 0, u64: 0 }; max_event as usize],
        })
    }

    /// timeout ms
    pub fn wait_events(&mut self, timeout: i32) -> Result<(), &'static str> {
        match self.epoll.wait(&mut self.events, timeout) {
            Ok(0) => return Ok(()),
            Ok(num) => {
                for event in self.events.iter().take(num as usize) {
                    if event.u64 == self.token {
                        loop {
                            match self.tcp_listen.accept() {
                                Ok((socket, socket_addr)) => {
                                    self.tcp_event.accept(socket, socket_addr)
                                }
                                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                    break;
                                }
                                Err(err) => {
                                    println!("tcp_listen.accept Error:{:?}", err);
                                    break;
                                }
                            }
                        }
                        continue;
                    }

                    if (event.events & libc::EPOLLIN as u32) != 0 {
                        self.tcp_event.read(event.u64);
                    }
                    if (event.events & libc::EPOLLOUT as u32) != 0 {
                        self.tcp_event.write(event.u64);
                    }
                    if (event.events & libc::EPOLLERR as u32) != 0 {
                        self.tcp_event.error(event.u64);
                    }
                    //if event.events & libc::EPOLLHUP {}  | libc::EPOLLHUP
                }
                return Ok(());
            }
            Err(err) => return Err(err),
        }
    }
}
