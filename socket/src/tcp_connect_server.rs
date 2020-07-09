use crate::epoll::Epoll;
use crate::message::MsgData;
use crate::message::MsgDataId;
use crate::message::NetMsg;
use crate::tcp_server_config::TcpServerConfig;
use crate::tcp_socket::ReadResult;
use crate::tcp_socket::WriteResult;
use crate::tcp_socket_mgmt::TcpSocketMgmt;
use libc;
use log::{error, warn};
use std::io::Error;
use std::io::ErrorKind;
use std::net::Shutdown;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

use std::thread;

use crate::tcp_socket::TcpSocket;

const TCP_LISTEN_ID: u64 = 0;
const EPOLL_IN_OUT: i32 = (libc::EPOLLOUT | libc::EPOLLIN) as i32;

pub struct TcpConnectServer<'a> {
    epoll: Epoll,
    tcp_socket_mgmt: TcpSocketMgmt,
    net_msg_cb: &'a mut dyn Fn(NetMsg),
    vec_epoll_event: Vec<libc::epoll_event>,
}

impl<'a> Drop for TcpConnectServer<'a> {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped TcpConnectServer while unwinding");
        } else {
            error!("dropped TcpConnectServer while not unwinding");
        }
    }
}

impl<'a> TcpConnectServer<'a> {
    pub fn new(cfg: &TcpServerConfig, net_msg_cb: &'a mut dyn Fn(NetMsg)) -> Result<Self, String> {
        let epoll: Epoll = Epoll::new()?;
        let tcp_socket_mgmt = TcpSocketMgmt::new(
            TCP_LISTEN_ID,
            cfg.max_socket,
            cfg.msg_max_size,
            cfg.wait_write_msg_max_num,
        )?;

        Ok(TcpConnectServer {
            epoll,
            net_msg_cb,
            tcp_socket_mgmt,
            vec_epoll_event: vec![
                libc::epoll_event { events: 0, u64: 0 };
                cfg.epoll_max_events as usize
            ],
        })
    }
    pub fn tick(&mut self) {}
    pub fn epoll_event(&mut self, epoll_wait_timeout: i32) -> Result<u16, String> {
        match self
            .epoll
            .wait(epoll_wait_timeout, &mut self.vec_epoll_event)
        {
            Ok(0) => return Ok(0),
            Ok(num) => {
                for n in 0..num as usize {
                    let event = self.vec_epoll_event[n];
                    if (event.events & libc::EPOLLIN as u32) != 0 {
                        self.read_event(event.u64);
                    }
                    if (event.events & libc::EPOLLOUT as u32) != 0 {
                        self.write_event(event.u64);
                    }
                    if (event.events & libc::EPOLLERR as u32) != 0 {
                        self.error_event(event.u64, Error::last_os_error().to_string());
                    }
                }
                return Ok(num as u16);
            }
            Err(err) => return Err(err),
        }
    }

    fn read_event(&mut self, id: u64) {
        if let Some(tcp_socket) = self.tcp_socket_mgmt.get_tcp_socket(id) {
            loop {
                match tcp_socket.reader.read(&mut tcp_socket.socket) {
                    ReadResult::Data(msg_data) => {
                        (self.net_msg_cb)(NetMsg {
                            id: id,
                            data: msg_data,
                        });
                    }
                    ReadResult::BufferIsEmpty => {
                        break;
                    }
                    ReadResult::ReadZeroSize => {
                        self.del_socket(id);
                        warn!("tcp_socket.reader.read :{}", "Read Zero Size");
                        break;
                    }
                    ReadResult::Error(err) => {
                        self.del_socket(id);
                        error!("tcp_socket.reader.read id:{} err:{}", id, err);
                        break;
                    }
                }
            }
        } else {
            warn!("read_event tcp_socket_mgmt id no exitis:{}", id);
        }
    }

    #[inline]
    pub fn write_net_msg(&mut self, net_msg: NetMsg) {
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
                if tcp_socket.writer.get_msg_data_count() == 1 {
                    if let Err(err) = write_data(&self.epoll, net_msg.id, tcp_socket) {
                        self.del_socket(net_msg.id);
                        warn!("tcp_socket.writer.write err:{}", err);
                    }
                }
            }
            None => {
                warn!("net_msg.id no exitis:{}", net_msg.id);
            }
        }
    }

    fn write_event(&mut self, id: u64) {
        if let Some(tcp_socket) = self.tcp_socket_mgmt.get_tcp_socket(id) {
            if let Err(err) = write_data(&self.epoll, id, tcp_socket) {
                self.del_socket(id);
                warn!("tcp_socket.writer.write err:{}", err);
            }
        } else {
            warn!("write_event tcp_socket_mgmt id no exitis:{}", id);
        }
    }

    fn error_event(&mut self, id: u64, err: String) {
        self.del_socket(id);
        warn!("epoll event error:{}", err);
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
                    warn!("epoll.ctl_del_fd({}) Error:{}", id, err);
                }
                (self.net_msg_cb)(NetMsg {
                    id: id,
                    data: Box::new(MsgData {
                        ext: 0,
                        data: vec![],
                        pid: MsgDataId::SocketClose as u16,
                    }),
                });
            }
            Err(err) => {
                warn!("tcp_socket_mgmt.del_socket({}) Error:{}", id, err);
            }
        }
    }
}

fn write_data(epoll: &Epoll, id: u64, tcp_socket: &mut TcpSocket) -> Result<(), String> {
    match tcp_socket.writer.write(&mut tcp_socket.socket) {
        WriteResult::Finish => {
            if tcp_socket.events == libc::EPOLLIN {
                return Ok(());
            }
            tcp_socket.events = libc::EPOLLIN;
            return epoll.ctl_mod_fd(id, tcp_socket.socket.as_raw_fd(), libc::EPOLLIN);
        }
        WriteResult::BufferFull => {
            if tcp_socket.events == EPOLL_IN_OUT {
                return Ok(());
            }
            tcp_socket.events = EPOLL_IN_OUT;
            return epoll.ctl_mod_fd(id, tcp_socket.socket.as_raw_fd(), EPOLL_IN_OUT);
        }
        WriteResult::Error(err) => return Err(err),
    }
}
