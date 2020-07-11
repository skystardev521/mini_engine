use crate::message::MsgData;
use crate::message::NetMsg;
use crate::message::ProtoId;
use crate::os_epoll::OSEpoll;
use crate::os_socket;
use crate::tcp_connect::TcpConnect;
use crate::tcp_connect_config::TcpConnectConfig;
use crate::tcp_connect_mgmt::TcpConnectMgmt;
use crate::tcp_socket_reader::ReadResult;
use crate::tcp_socket_writer::WriteResult;
use libc;
use log::{error, info, warn};
use std::io::Error;
use std::io::ErrorKind;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::time::Duration;
use utils::time;

use std::thread;

use crate::tcp_socket::TcpSocket;

const EPOLL_IN_OUT: i32 = (libc::EPOLLOUT | libc::EPOLLIN) as i32;

pub struct TcpConnectServer<'a> {
    os_epoll: OSEpoll,
    config: &'a TcpConnectConfig,
    tcp_connect_mgmt: TcpConnectMgmt<'a>,
    vec_epoll_event: Vec<libc::epoll_event>,
    net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
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
    pub fn new(
        config: &'a TcpConnectConfig,
        net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
    ) -> Result<Self, String> {
        let os_epoll: OSEpoll = OSEpoll::new()?;

        let tcp_connect_mgmt = TcpConnectMgmt::new(config)?;

        Ok(TcpConnectServer {
            os_epoll,
            config,
            net_msg_cb,
            tcp_connect_mgmt,
            vec_epoll_event: vec![
                libc::epoll_event { events: 0, u64: 0 };
                config.epoll_max_events as usize
            ],
        })
    }
    pub fn tick(&mut self) {}
    pub fn epoll_event(&mut self, epoll_wait_timeout: i32) -> Result<u16, String> {
        match self
            .os_epoll
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

    fn read_event(&mut self, sid: u64) {
        //info!("read id:{}", id);
        if let Some(tcp_connect) = self.tcp_connect_mgmt.get_tcp_connect(sid) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                let mut msg_cb_err_reason = None;
                loop {
                    match tcp_socket.reader.read(&mut tcp_socket.socket) {
                        ReadResult::Data(msg_data) => {
                            match (self.net_msg_cb)(NetMsg {
                                sid: sid,
                                data: msg_data,
                            }) {
                                Ok(()) => (),
                                Err(pid) => {
                                    msg_cb_err_reason = Some(pid);
                                }
                            }
                        }
                        ReadResult::BufferIsEmpty => {
                            break;
                        }
                        ReadResult::ReadZeroSize => {
                            epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                            warn!("tcp_socket.reader.read :{}", "Read Zero Size");
                            break;
                        }
                        ReadResult::Error(err) => {
                            epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                            error!("tcp_socket.reader.read id:{} err:{}", sid, err);
                            break;
                        }
                    }
                }
                if let Some(pid) = msg_cb_err_reason {
                    self.write_net_msg(NetMsg {
                        sid: sid,
                        data: Box::new(MsgData {
                            ext: 0,
                            data: vec![],
                            pid: pid as u16,
                        }),
                    });
                }
            }
        } else {
            warn!("read_event tcp_connect_mgmt id no exitis:{}", sid);
        };
    }

    #[inline]
    pub fn write_net_msg(&mut self, net_msg: NetMsg) {
        match self.tcp_connect_mgmt.get_tcp_connect(net_msg.sid) {
            Some(tcp_connect) => {
                if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                    if tcp_socket.writer.get_msg_data_count() > self.config.wait_write_msg_max_num {
                        // 发送服务繁忙消息

                        warn!("net_msg.id:{} Too much msg_data not send", net_msg.sid);
                        self.send_simple_net_msg(net_msg.sid, ProtoId::BusyServer);
                        return;
                    }
                    match tcp_socket.writer.add_msg_data(net_msg.data) {
                        Ok(()) => (),
                        Err(err) => {
                            error!("tcp_socket.writer.add_msg_data error:{}", err);
                        }
                    }
                    if tcp_socket.writer.get_msg_data_count() == 1 {
                        if let Err(err) = write_data(&self.os_epoll, net_msg.sid, tcp_socket) {
                            warn!("tcp_socket.writer.write err:{}", err);

                            epoll_del_fd(
                                &self.os_epoll,
                                net_msg.sid,
                                tcp_socket.socket.as_raw_fd(),
                            );

                            match reconnect_tcp_connect(self.config, &tcp_connect) {
                                Ok(None) => (),
                                Ok(Some(new_tcp_socket)) => {
                                    tcp_connect.set_tcp_socket_opt(Some(new_tcp_socket));
                                }
                                Err(err) => {
                                    tcp_connect.set_tcp_socket_opt(None);
                                    error!(
                                        "reconnect_tcp_connect {} error:{}",
                                        tcp_connect.get_socket_addr(),
                                        err
                                    )
                                }
                            }
                        }
                    }
                }
            }

            None => {
                warn!("net_msg.sid no exitis:{}", net_msg.sid);
                self.send_simple_net_msg(net_msg.sid, ProtoId::SocketClose);
            }
        }
    }

    fn write_event(&mut self, sid: u64) {
        if let Some(tcp_connect) = self.tcp_connect_mgmt.get_tcp_connect(sid) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                if let Err(err) = write_data(&self.os_epoll, sid, tcp_socket) {
                    warn!("tcp_socket.writer.write sid:{} err:{}", sid, err);
                    epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());

                    match reconnect_tcp_connect(self.config, &tcp_connect) {
                        Ok(None) => (),
                        Ok(Some(new_tcp_socket)) => {
                            tcp_connect.set_tcp_socket_opt(Some(new_tcp_socket));
                        }
                        Err(err) => {
                            tcp_connect.set_tcp_socket_opt(None);
                            error!(
                                "reconnect_tcp_connect {} error:{}",
                                tcp_connect.get_socket_addr(),
                                err
                            )
                        }
                    }
                }
            }
        } else {
            warn!("write_event tcp_connect_mgmt id no exitis:{}", sid);
        }
    }

    fn error_event(&mut self, sid: u64, err: String) {
        warn!("os_epoll error event:{}", err);
        if let Some(tcp_connect) = self.tcp_connect_mgmt.get_tcp_connect(sid) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
            }

            match reconnect_tcp_connect(self.config, &tcp_connect) {
                Ok(None) => (),
                Ok(Some(new_tcp_socket)) => {
                    tcp_connect.set_tcp_socket_opt(Some(new_tcp_socket));
                }
                Err(err) => {
                    tcp_connect.set_tcp_socket_opt(None);
                    error!(
                        "reconnect_tcp_connect {} error:{}",
                        tcp_connect.get_socket_addr(),
                        err
                    )
                }
            }
        }
    }

    fn send_simple_net_msg(&mut self, sid: u64, pid: ProtoId) {
        match (self.net_msg_cb)(NetMsg {
            sid: sid,
            data: Box::new(MsgData {
                ext: 0,
                data: vec![],
                pid: pid as u16,
            }),
        }) {
            Ok(()) => (),
            Err(pid) => {
                warn!("send_simple_net_msg ({}) net_msg_cb return :{:?}", sid, pid);
            }
        }
    }
}

fn epoll_del_fd(os_epoll: &OSEpoll, sid: u64, raw_fd: RawFd) {
    if let Err(err) = os_epoll.ctl_del_fd(sid, raw_fd) {
        warn!("os_epoll.ctl_del_fd({}) Error:{}", sid, err);
    }
}

fn write_data(os_epoll: &OSEpoll, sid: u64, tcp_socket: &mut TcpSocket) -> Result<(), String> {
    match tcp_socket.writer.write(&mut tcp_socket.socket) {
        WriteResult::Finish => {
            if tcp_socket.epoll_events == libc::EPOLLIN {
                return Ok(());
            }
            tcp_socket.epoll_events = libc::EPOLLIN;
            return os_epoll.ctl_mod_fd(sid, tcp_socket.socket.as_raw_fd(), libc::EPOLLIN);
        }
        WriteResult::BufferFull => {
            if tcp_socket.epoll_events == EPOLL_IN_OUT {
                return Ok(());
            }
            tcp_socket.epoll_events = EPOLL_IN_OUT;
            return os_epoll.ctl_mod_fd(sid, tcp_socket.socket.as_raw_fd(), EPOLL_IN_OUT);
        }
        WriteResult::Error(err) => return Err(err),
    }
}

pub fn reconnect_tcp_connect(
    config: &TcpConnectConfig,
    tcp_connect: &TcpConnect,
) -> Result<Option<TcpSocket>, String> {
    let now_timestamp = time::timestamp();
    if tcp_connect.get_last_reconnect_timestamp() + config.reconnect_socket_interval as u64
        > now_timestamp
    {
        return Ok(None);
    }
    match new_tcp_socket(tcp_connect.get_socket_addr(), config) {
        Ok(tcp_socket) => {
            return Ok(Some(tcp_socket));
        }
        Err(err) => return Err(err),
    }
}

fn new_tcp_socket(socket_addr: &String, config: &TcpConnectConfig) -> Result<TcpSocket, String> {
    match socket_addr.parse::<SocketAddr>() {
        Ok(addr) => {
            let duration = Duration::from_millis(config.connect_timeout_duration as u64);
            match TcpStream::connect_timeout(&addr, duration) {
                Ok(socket) => {
                    if let Err(err) = socket.set_nonblocking(true) {
                        return Err(format!("{}", err));
                    }
                    if let Err(err) = socket.set_nodelay(config.tcp_nodelay_value) {
                        return Err(format!("set_nodelay:{}", err));
                    }

                    let raw_fd = socket.as_raw_fd();
                    os_socket::setsockopt(
                        raw_fd,
                        libc::SOL_SOCKET,
                        libc::SO_RCVBUF,
                        config.socket_read_buffer,
                    )?;

                    os_socket::setsockopt(
                        raw_fd,
                        libc::SOL_SOCKET,
                        libc::SO_SNDBUF,
                        config.socket_write_buffer,
                    )?;

                    return Ok(TcpSocket::new(socket, config.msg_max_size));
                }
                Err(err) => return Err(format!("{}", err)),
            }
        }
        Err(err) => Err(format!("{}", err)),
    }
}
