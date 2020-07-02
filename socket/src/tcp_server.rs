use crate::epoll::Epoll;
use crate::message::NetMsg;
use crate::tcp_bind::TcpBind;
use crate::tcp_server_config::TcpServerConfig;
use crate::tcp_socket::ReadResult;
use crate::tcp_socket::WriteResult;
use crate::tcp_socket_mgmt::TcpSocketMgmt;
use libc;
use log::{error, warn};
use std::io::ErrorKind;
use std::net::Shutdown;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use utils::native;

const TCP_BIND_ID: u64 = 0;
pub struct TcpServer {
    epoll: Epoll,
    tcp_bind: TcpBind,
    epoll_wait_timeout: i32,
    tcp_socket_mgmt: TcpSocketMgmt,
    vec_epoll_event: Vec<libc::epoll_event>,
}

impl TcpServer {
    pub fn new(cfg: &TcpServerConfig) -> Result<Self, String> {
        let epoll: Epoll = Epoll::new()?;
        let tcp_bind = TcpBind::new(&cfg.bind_socket_addr)?;
        let tcp_socket_mgmt = TcpSocketMgmt::new(
            TCP_BIND_ID,
            cfg.max_socket,
            cfg.msg_max_size,
            cfg.wait_write_msg_max_num,
        )?;

        Ok(TcpServer {
            epoll,
            tcp_bind,
            tcp_socket_mgmt,
            epoll_wait_timeout: cfg.epoll_wait_timeout,
            vec_epoll_event: vec![
                libc::epoll_event { events: 0, u64: 0 };
                cfg.epoll_max_events as usize
            ],
        })
    }
    pub fn run(&mut self) {
        /*
        loop {
            self.epoll_wait();
        }
        */
    }

    #[inline]
    pub fn write_msg_data(&mut self, net_msg: NetMsg) {
        let msg_max_num = self.tcp_socket_mgmt.get_wait_write_msg_max_num();
        match self.tcp_socket_mgmt.get_tcp_socket(net_msg.id) {
            Some(tcp_socket) => {
                if tcp_socket.writer.get_msg_data_count() > msg_max_num {
                    self.del_socket(net_msg.id);
                    warn!("net_msg.id:{} Too much msg_data not send", net_msg.id);
                    return;
                }
                match tcp_socket.writer.add_msg_data(net_msg.data) {
                    Ok(()) => (),
                    Err(err) => {
                        error!("tcp_socket.writer.add_msg_data error:{}", err);
                    }
                }
            }

            None => {
                warn!("net_msg.id no exitis:{}", net_msg.id);
            }
        }
    }
    pub fn epoll_wait(&mut self) -> Result<u32, String> {
        match self
            .epoll
            .wait(self.epoll_wait_timeout, &mut self.vec_epoll_event)
        {
            Ok(0) => return Ok(0),
            Ok(num) => {
                for n in 0..num as usize {
                    let event = self.vec_epoll_event[n];
                    if event.u64 == TCP_BIND_ID {
                        self.accept();
                        continue;
                    }

                    if (event.events & libc::EPOLLIN as u32) != 0 {
                        self.read(event.u64);
                    }
                    if (event.events & libc::EPOLLOUT as u32) != 0 {
                        self.write(event.u64);
                    }
                    if (event.events & libc::EPOLLERR as u32) != 0 {
                        self.error(event.u64, native::c_strerr());
                    }
                    //if event.events & libc::EPOLLHUP {}  | libc::EPOLLHUP
                }
                return Ok(num);
            }
            Err(err) => return Err(err),
        }
    }
    fn read(&mut self, id: u64) {
        if let Some(tcp_socket) = self.tcp_socket_mgmt.get_tcp_socket(id) {
            loop {
                match tcp_socket.reader.read(&mut tcp_socket.socket) {
                    ReadResult::Data(msg_data) => {
                        /*
                        (self.net_msg_cb)(NetMsg {
                            id: id,
                            data: msg_data,
                        });
                        */
                    }
                    ReadResult::BufferIsEmpty => {
                        break;
                    }
                    ReadResult::ReadZeroSize => {
                        self.del_socket(id);
                        warn!("tcp_socket.reader.read :{}", "ReadZeroSize");
                        break;
                    }
                    ReadResult::Error(err) => {
                        self.del_socket(id);
                        error!("tcp_socket.reader.read err:{}", err);
                        break;
                    }
                }
            }
        }
    }

    fn write(&mut self, id: u64) {
        if let Some(tcp_socket) = self.tcp_socket_mgmt.get_tcp_socket(id) {
            match tcp_socket.writer.write(&mut tcp_socket.socket) {
                WriteResult::Finish => (),
                WriteResult::BufferFull => {
                    match self.epoll.ctl_mod_fd(
                        id,
                        tcp_socket.socket.as_raw_fd(),
                        (libc::EPOLLOUT | libc::EPOLLIN) as i32,
                    ) {
                        Ok(()) => (),
                        Err(err) => {
                            self.del_socket(id);
                            warn!("tcp_socket.writer.write epoll.ctl_mod_fd err:{}", err);
                        }
                    }
                }
                WriteResult::Error(err) => {
                    self.del_socket(id);
                    warn!("tcp_socket.writer.write err:{}", err);
                }
            }
        }
    }

    fn error(&mut self, id: u64, err: String) {
        self.del_socket(id);
        warn!("epoll event error:{}", err);
    }

    fn accept(&mut self) {
        loop {
            match self.tcp_bind.get_listen().accept() {
                Ok((socket, _)) => {
                    self.new_socket(socket);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(ref err) => error!("tcp listen accept() error:{}", err),
            }
        }
    }
    fn new_socket(&mut self, socket: TcpStream) {
        match socket.set_nonblocking(true) {
            Ok(()) => {
                let raw_fd = socket.as_raw_fd();
                match self.tcp_socket_mgmt.new_socket(socket) {
                    Ok(socket_id) => {
                        match self.epoll.ctl_add_fd(socket_id, raw_fd, libc::EPOLLIN) {
                            Ok(()) => (),
                            Err(err) => {
                                error!("epoll ctl_add_fd error:{}", err);
                            }
                        };
                    }
                    Err(err) => {
                        error!("new_socket:{}", err);
                    }
                }
            }
            Err(err) => {
                error!("set_nonblocking:{}", err);
                match socket.shutdown(Shutdown::Both) {
                    Ok(()) => (),
                    Err(err) => {
                        error!("accept socket shutdown:{}", err);
                    }
                }
            }
        }
    }

    fn del_socket(&mut self, id: u64) {
        match self.tcp_socket_mgmt.del_socket(id) {
            Ok(tcp_socket) => {
                if let Err(err) = self.epoll.ctl_del_fd(id, tcp_socket.socket.as_raw_fd()) {
                    warn!(
                        "tcp_socket_mgmt.del_clientepoll.ctl_del_fd({}) Error:{}",
                        id, err
                    );
                }
            }
            Err(err) => {
                warn!("tcp_socket_mgmt.del_client({}) Error:{}", id, err);
            }
        }
    }
}
