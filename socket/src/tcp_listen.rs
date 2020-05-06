use libc;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

use crate::epoll::Epoll;
use crate::tcp_event::TcpEvent;
use crate::utils;

pub struct TcpListen {
    token: u64,
    epoll: Epoll,
    tcp_event: TcpEvent,
    tcp_listen: TcpListener,
    events: Vec<libc::epoll_event>,
    accept: fn(TcpStream, SocketAddr),
}

impl TcpListen {
    pub fn new(
        token: u64,
        events: u16,
        tcp_event: TcpEvent,
        socekt_addr: &String,
    ) -> Result<Self, String> {
        let mut max_event = events;
        if max_event < 16 {
            max_event = 16
        }

        let epoll = match Epoll::new() {
            Ok(epoll) => epoll,
            Err(err) => return Err(err),
        };

        let tcp_listen = match TcpListener::bind(socekt_addr) {
            Ok(tcp_listen) => tcp_listen,
            Err(err) => return Err(utils::error_kind_string(err.kind())),
        };

        match &tcp_listen.set_nonblocking(true) {
            Ok(()) => (),
            Err(err) => return Err(utils::error_kind_string(err.kind())),
        }

        match utils::setsockopt(tcp_listen.as_raw_fd(), libc::SO_REUSEADDR, 1) {
            Ok(()) => (),
            Err(err) => return Err(err),
        }

        epoll.ctl_add_fd(net_token as u64, tcp_listen.as_raw_fd(), libc::EPOLLIN)?;

        Ok(Listen {
            epoll: epoll,
            tcp_event: tcp_event,
            net_token: net_token,
            tcp_listen: tcp_listen,
            events: vec![libc::epoll_event { events: 0, u64: 0 }; max_event as usize],
        })
    }
    ///ms
    pub fn wait_events(&mut self, timeout: i32) -> Result<(), String> {
        let n = self.epoll.wait(&mut self.events, timeout);
        if n == 0 {
            return Ok(());
        }
        if n == -1 {
            return Err(utils::c_err_string());
        }
        for event in self.events.iter().take(n as usize) {
            if event.u64 == self.net_token {
                loop {
                    match self.tcp_listen.accept() {
                        Ok((socket, socket_addr)) => (self.accept)(socket, socket_addr),
                        Err(_) => break,
                    }
                }
                continue;
            }

            if (event.events & libc::EPOLLIN as u32) != 0 {
                (self.recv)(event.u64);
            }
            if (event.events & libc::EPOLLOUT as u32) != 0 {
                (self.send)(event.u64);
            }
            if (event.events & libc::EPOLLERR as u32) != 0 {
                (self.error)(event.u64);
            }
            //if event.events & libc::EPOLLHUP {}  | libc::EPOLLHUP
        }
        Ok(())
    }
}
