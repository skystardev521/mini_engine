use crate::message::MsgData;
use crate::message::NetMsg;
use crate::message::ProtoId;
use crate::os_epoll::OSEpoll;
use crate::os_socket;
use crate::tcp_connect::TcpConnect;
use crate::tcp_connect_config::TcpConnectConfig;
use crate::tcp_socket_reader::ReadResult;
use crate::tcp_socket_writer::WriteResult;
use libc;
use log::{debug, error, warn};
use mini_utils::time;
use std::io::Error;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::time::Duration;

use std::thread;

use crate::tcp_socket::TcpSocket;

const EPOLL_IN_OUT: i32 = (libc::EPOLLOUT | libc::EPOLLIN) as i32;

pub struct TcpConnectService<'a> {
    os_epoll: OSEpoll,
    epoll_max_events: u16,
    vec_tcp_connect: Vec<TcpConnect>,
    vec_epoll_event: Vec<libc::epoll_event>,
    net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
}

impl<'a> Drop for TcpConnectService<'a> {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped TcpConnectService while unwinding");
        } else {
            error!("dropped TcpConnectService while not unwinding");
        }
    }
}

impl<'a> TcpConnectService<'a> {
    pub fn new(
        vec_tcp_connect_config: Vec<TcpConnectConfig>,
        net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
    ) -> Result<Self, String> {
        let os_epoll: OSEpoll = OSEpoll::new()?;
        let tcp_connect_num = vec_tcp_connect_config.len();

        let vec_tcp_connect = init_tcp_connect(&os_epoll, vec_tcp_connect_config);

        Ok(TcpConnectService {
            os_epoll,
            net_msg_cb,
            vec_tcp_connect,
            epoll_max_events: tcp_connect_num as u16,
            vec_epoll_event: vec![libc::epoll_event { events: 0, u64: 0 }; tcp_connect_num],
        })
    }

    pub fn tick(&mut self) {
        self.check_connect();
    }

    pub fn get_conn_info(&self) -> Vec<(u16, String)> {
        let len = self.vec_tcp_connect.len();
        let mut vec_info = Vec::with_capacity(len);
        for i in 0..len {
            vec_info.push((i as u16, self.vec_tcp_connect[i].get_config().name.clone()));
        }
        vec_info
    }

    fn check_connect(&mut self) {
        for tcp_connect in &mut self.vec_tcp_connect {
            if tcp_connect.get_tcp_socket_opt().is_none() {
                tcp_connect.set_tcp_socket_opt(tcp_reconnect(&self.os_epoll, tcp_connect));
            }
        }
    }

    pub fn get_epoll_max_events(&self) -> u16 {
        self.epoll_max_events
    }

    pub fn epoll_event(&mut self, epoll_wait_timeout: i32) -> Result<u16, String> {
        match self
            .os_epoll
            .wait(epoll_wait_timeout, &mut self.vec_epoll_event)
        {
            Ok(0) => return Ok(0),
            Ok(epevs) => {
                for n in 0..epevs as usize {
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
                return Ok(epevs as u16);
            }
            Err(err) => return Err(err),
        }
    }

    fn read_event(&mut self, sid: u64) {
        //info!("read id:{}", id);
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(sid as usize) {
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
                            warn!("tcp_socket.reader.read :{}", "Read Zero Size");
                            epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                            tcp_connect
                                .set_tcp_socket_opt(tcp_reconnect(&self.os_epoll, &tcp_connect));
                            break;
                        }
                        ReadResult::Error(err) => {
                            error!("tcp_socket.reader.read id:{} err:{}", sid, err);
                            epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                            tcp_connect
                                .set_tcp_socket_opt(tcp_reconnect(&self.os_epoll, &tcp_connect));
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
        match self.vec_tcp_connect.get_mut(net_msg.sid as usize) {
            Some(tcp_connect) => {
                let max_num = tcp_connect.get_config().wait_write_msg_max_num;
                if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                    if tcp_socket.writer.get_msg_data_count() > max_num {
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

                            tcp_connect
                                .set_tcp_socket_opt(tcp_reconnect(&self.os_epoll, &tcp_connect));
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
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(sid as usize) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                if let Err(err) = write_data(&self.os_epoll, sid, tcp_socket) {
                    warn!("tcp_socket.writer.write sid:{} err:{}", sid, err);
                    epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                    tcp_connect.set_tcp_socket_opt(tcp_reconnect(&self.os_epoll, &tcp_connect));
                }
            }
        } else {
            warn!("write_event tcp_connect_mgmt id no exitis:{}", sid);
        }
    }

    fn error_event(&mut self, sid: u64, err: String) {
        warn!("os_epoll error event:{}", err);
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(sid as usize) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
            }
            tcp_connect.set_tcp_socket_opt(tcp_reconnect(&self.os_epoll, &tcp_connect));
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

fn init_tcp_connect(
    os_epoll: &OSEpoll,
    vec_tcp_connect_config: Vec<TcpConnectConfig>,
) -> Vec<TcpConnect> {
    let mut id = 0;
    let connect_num = vec_tcp_connect_config.len();
    let mut vec_tcp_connect = Vec::with_capacity(connect_num);
    for connect_config in vec_tcp_connect_config {
        match new_tcp_socket(os_epoll, id, &connect_config) {
            Ok(tcp_socket) => {
                vec_tcp_connect.push(TcpConnect::new(id, connect_config, Some(tcp_socket)));
            }
            Err(err) => {
                error!(
                    "tcp_connect addr {} error:{}",
                    connect_config.socket_addr, err
                );
                vec_tcp_connect.push(TcpConnect::new(id, connect_config, None));
            }
        }
        id += 1;
    }
    vec_tcp_connect
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

/// 断线重连
fn tcp_reconnect(os_epoll: &OSEpoll, tcp_connect: &TcpConnect) -> Option<TcpSocket> {
    let ts = time::timestamp();
    if tcp_connect.get_last_reconnect_timestamp()
        + tcp_connect.get_config().reconnect_interval as u64
        > ts
    {
        return None;
    }
    tcp_connect.set_last_reconnect_timestamp(ts);
    match new_tcp_socket(os_epoll, tcp_connect.get_id(), tcp_connect.get_config()) {
        Ok(tcp_socket) => {
            return Some(tcp_socket);
        }
        Err(err) => {
            error!(
                "tcp_reconnect {} error:{}",
                tcp_connect.get_config().socket_addr,
                err
            );
            return None;
        }
    }
}

/// 新建链接
fn new_tcp_socket(
    os_epoll: &OSEpoll,
    sid: u64,
    config: &TcpConnectConfig,
) -> Result<TcpSocket, String> {
    match config.socket_addr.parse::<SocketAddr>() {
        Ok(addr) => {
            let duration = Duration::from_millis(config.connect_timeout_duration as u64);
            match TcpStream::connect_timeout(&addr, duration) {
                Ok(socket) => {
                    if let Err(err) = socket.set_nonblocking(true) {
                        return Err(format!("set_nonblocking:{}", err.to_string()));
                    }

                    if let Err(err) = socket.set_nodelay(config.tcp_nodelay_value) {
                        return Err(format!("set_tcp_nodelay:{}", err));
                    }

                    let raw_fd = socket.as_raw_fd();
                    if let Err(err) = os_epoll.ctl_add_fd(sid, raw_fd, libc::EPOLLIN) {
                        return Err(format!("os_epoll ctl_add_fd error:{}", err));
                    }

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
                    debug!("connect:{} success", config.socket_addr);
                    return Ok(TcpSocket::new(socket, config.msg_max_size));
                }
                Err(err) => return Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
