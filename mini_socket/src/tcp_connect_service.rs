use crate::tcp_socket_msg::SProtoId;
use crate::os_epoll::OSEpoll;
use crate::os_socket;
use crate::tcp_socket_rw::ReadResult;
use crate::tcp_socket_rw::TcpSocketRw;
use crate::tcp_socket_rw::WriteResult;

use crate::tcp_connect::TcpConnect;
use crate::tcp_connect_config::TcpConnectConfig;
use libc;
use log::{debug, error, warn};
use mini_utils::time;
use std::io::Error;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::thread;
use std::time::Duration;

use crate::tcp_socket::TcpSocket;

const EPOLL_IN_OUT: i32 = (libc::EPOLLOUT | libc::EPOLLIN) as i32;

pub struct TcpConnectService<'a, TBRW, MSG> {
    os_epoll: OSEpoll,
    share_buffer: Vec<u8>,
    epoll_max_events: u16,
    phantom: PhantomData<TBRW>,
    vec_tcp_connect: Vec<TcpConnect<MSG>>,
    vec_epoll_event: Vec<libc::epoll_event>,
    net_msg_cb_fn: &'a mut dyn Fn(u64, Vec<MSG>),
    exc_msg_cb_fn: &'a mut dyn Fn(u64, SProtoId),
}

impl<'a, TBRW, MSG> Drop for TcpConnectService<'a, TBRW, MSG> {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped TcpConnectService while unwinding");
        } else {
            error!("dropped TcpConnectService while not unwinding");
        }
    }
}

impl<'a, TBRW, MSG> TcpConnectService<'a, TBRW, MSG>
where
    TBRW: TcpSocketRw<MSG> + Default + 'static,
{
    pub fn new(
        vec_tcp_connect_config: Vec<TcpConnectConfig>,
        net_msg_cb_fn: &'a mut dyn Fn(u64, Vec<MSG>),
        exc_msg_cb_fn: &'a mut dyn Fn(u64, SProtoId),
    ) -> Result<Self, String> {
        let os_epoll: OSEpoll = OSEpoll::new()?;
        let tcp_connect_num = vec_tcp_connect_config.len();
        let share_buffer_size = Self::get_share_buffer_size(&vec_tcp_connect_config) * 3;
        let vec_tcp_connect = init_tcp_connect::<TBRW, MSG>(&os_epoll, vec_tcp_connect_config);

        Ok(TcpConnectService {
            os_epoll,
            net_msg_cb_fn,
            exc_msg_cb_fn,
            vec_tcp_connect,
            phantom: PhantomData,
            epoll_max_events: tcp_connect_num as u16,
            share_buffer: vec![0u8; share_buffer_size],
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
                tcp_connect
                    .set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(&self.os_epoll, tcp_connect));
            }
        }
    }

    pub fn get_epoll_max_events(&self) -> u16 {
        self.epoll_max_events
    }

    pub fn epoll_event(&mut self, wait_timeout: i32) -> Result<u32, String> {
        match self.os_epoll.wait(wait_timeout, &mut self.vec_epoll_event) {
            Ok(0) => Ok(0),
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
                Ok(epevs)
            }
            Err(err) => Err(err),
        }
    }

    fn read_event(&mut self, cid: u64) {
        //info!("read id:{}", id);
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(cid as usize) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                //loop {
                match tcp_socket.read(&mut self.share_buffer) {
                    ReadResult::Data(vec_msg) => {
                        (self.net_msg_cb_fn)(cid, vec_msg);
                    }
                    ReadResult::Error(vec_msg, err) => {
                        (self.net_msg_cb_fn)(cid, vec_msg);
                        error!("tcp_socket.read id:{} err:{}", cid, err);
                        epoll_del_fd(&self.os_epoll, cid, tcp_socket.socket.as_raw_fd());
                        tcp_connect.set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(
                            &self.os_epoll,
                            &tcp_connect,
                        ));
                    }
                }
            }
        //}
        } else {
            warn!("read_event tcp_connect_mgmt id no exitis:{}", cid);
        };
    }

    #[inline]
    pub fn write_msg(&mut self, cid: u64, msg: MSG) {
        match self.vec_tcp_connect.get_mut(cid as usize) {
            Some(tcp_connect) => {
                let msg_deque_size = tcp_connect.get_config().msg_deque_size;
                if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                    if tcp_socket.vec_queue_len() > msg_deque_size {
                        warn!("cid:{} Msg Queue Is Full", cid);
                        (self.exc_msg_cb_fn)(cid, SProtoId::MsgQueueFull);
                        return;
                    }

                    tcp_socket.push_vec_queue(msg);

                    if tcp_socket.vec_queue_len() == 1 {
                        if let Err(err) = write_data(&self.os_epoll, cid, tcp_socket) {
                            warn!("cid:{} write_data  err:{}", cid, err);
                            epoll_del_fd(&self.os_epoll, cid, tcp_socket.socket.as_raw_fd());
                            tcp_connect.set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(
                                &self.os_epoll,
                                &tcp_connect,
                            ));
                        }
                    }
                }
            }
            None => {
                warn!("write_msg socket id no exitis:{}", cid);
                (self.exc_msg_cb_fn)(cid, SProtoId::Disconnect);
            }
        }
    }

    fn write_event(&mut self, cid: u64) {
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(cid as usize) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                if let Err(err) = write_data(&self.os_epoll, cid, tcp_socket) {
                    warn!("tcp_socket.writer.write cid:{} err:{}", cid, err);
                    epoll_del_fd(&self.os_epoll, cid, tcp_socket.socket.as_raw_fd());
                    tcp_connect.set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(
                        &self.os_epoll,
                        &tcp_connect,
                    ));
                }
            }
        } else {
            error!("write_event cid no exit:{}", cid);
        }
    }

    fn error_event(&mut self, cid: u64, err: String) {
        warn!("error_event cid:{} error:{}", cid, err);
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(cid as usize) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                epoll_del_fd(&self.os_epoll, cid, tcp_socket.socket.as_raw_fd());
            }
            tcp_connect
                .set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(&self.os_epoll, &tcp_connect));
        }
    }

    fn get_share_buffer_size(vec_tcp_connect_config: &Vec<TcpConnectConfig>) -> usize {
        let mut buffer_size = 0;
        for cfg in vec_tcp_connect_config {
            if cfg.socket_read_buffer > buffer_size {
                buffer_size = cfg.socket_read_buffer;
            }
        }
        if buffer_size == 0{
            buffer_size = 1048576;
        }
        buffer_size as usize
    }
}


fn init_tcp_connect<TBRW, MSG>(
    os_epoll: &OSEpoll,
    vec_tcp_connect_config: Vec<TcpConnectConfig>,
) -> Vec<TcpConnect<MSG>>
where
    TBRW: TcpSocketRw<MSG> + Default + 'static,
{
    let mut id = 0;
    let connect_num = vec_tcp_connect_config.len();
    let mut vec_tcp_connect = Vec::with_capacity(connect_num);
    for connect_config in vec_tcp_connect_config {
        match new_tcp_socket::<TBRW, MSG>(os_epoll, id, &connect_config) {
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

fn epoll_del_fd(os_epoll: &OSEpoll, cid: u64, raw_fd: RawFd) {
    if let Err(err) = os_epoll.ctl_del_fd(cid, raw_fd) {
        warn!("os_epoll.ctl_del_fd({}) Error:{}", cid, err);
    }
}

fn write_data<MSG>(
    os_epoll: &OSEpoll,
    cid: u64,
    tcp_socket: &mut TcpSocket<MSG>,
) -> Result<(), String> {
    match tcp_socket.write() {
        WriteResult::Finish => {
            if tcp_socket.epevs == libc::EPOLLIN {
                return Ok(());
            }
            tcp_socket.epevs = libc::EPOLLIN;
            return os_epoll.ctl_mod_fd(cid, tcp_socket.socket.as_raw_fd(), libc::EPOLLIN);
        }
        WriteResult::BufferFull => {
            if tcp_socket.epevs == EPOLL_IN_OUT {
                return Ok(());
            }
            tcp_socket.epevs = EPOLL_IN_OUT;
            return os_epoll.ctl_mod_fd(cid, tcp_socket.socket.as_raw_fd(), EPOLL_IN_OUT);
        }
        WriteResult::Error(err) => return Err(err),
    }
}

/// 断线重连
fn tcp_reconnect<TBRW, MSG>(
    os_epoll: &OSEpoll,
    tcp_connect: &TcpConnect<MSG>,
    //tcp_socket_rw: Box<dyn TcpSocketRw<MSG>>,
) -> Option<TcpSocket<MSG>>
where
    TBRW: TcpSocketRw<MSG> + Default + 'static,
{
    let ts = time::timestamp();
    if tcp_connect.get_last_reconnect_timestamp()
        + tcp_connect.get_config().reconnect_interval as u64
        > ts
    {
        return None;
    }

    tcp_connect.set_last_reconnect_timestamp(ts);
    match new_tcp_socket::<TBRW, MSG>(os_epoll, tcp_connect.get_cid(), tcp_connect.get_config()) {
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
fn new_tcp_socket<TBRW, MSG>(
    os_epoll: &OSEpoll,
    cid: u64,
    config: &TcpConnectConfig,
) -> Result<TcpSocket<MSG>, String>
where
    TBRW: TcpSocketRw<MSG> + Default + 'static,
{
    match config.socket_addr.parse::<SocketAddr>() {
        Ok(addr) => {
            let duration = Duration::from_millis(config.connect_timeout_duration as u64);
            match TcpStream::connect_timeout(&addr, duration) {
                Ok(socket) => {
                    if let Err(err) = socket.set_nonblocking(true) {
                        return Err(format!("set_nonblocking:{}", err.to_string()));
                    }

                    if let Err(err) = socket.set_nodelay(config.tcp_nodelay) {
                        return Err(format!("set_tcp_nodelay:{}", err));
                    }

                    let raw_fd = socket.as_raw_fd();
                    if let Err(err) = os_epoll.ctl_add_fd(cid, raw_fd, libc::EPOLLIN) {
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
                    return Ok(TcpSocket::new(socket, Box::new(TBRW::default())));
                }
                Err(err) => return Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
