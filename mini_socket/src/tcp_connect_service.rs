use crate::message::ErrMsg;
use crate::os_epoll::OSEpoll;
use crate::os_socket;
use crate::tcp_buf_rw::ReadResult;
use crate::tcp_buf_rw::TcpBufRw;
use crate::tcp_buf_rw::WriteResult;
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
    vec_shared: Vec<u8>,
    epoll_max_events: u16,
    phantom: PhantomData<TBRW>,
    vec_tcp_connect: Vec<TcpConnect<MSG>>,
    vec_epoll_event: Vec<libc::epoll_event>,
    net_msg_cb_fn: &'a mut dyn Fn(u64, MSG),
    err_msg_cb_fn: &'a mut dyn Fn(u64, ErrMsg),
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
    TBRW: TcpBufRw<MSG> + Default + 'static,
{
    pub fn new(
        vec_tcp_connect_config: Vec<TcpConnectConfig>,
        net_msg_cb_fn: &'a mut dyn Fn(u64, MSG),
        err_msg_cb_fn: &'a mut dyn Fn(u64, ErrMsg),
    ) -> Result<Self, String> {
        let os_epoll: OSEpoll = OSEpoll::new()?;
        let tcp_connect_num = vec_tcp_connect_config.len();
        let vec_shared_size = get_max_socket_read_buffer(&vec_tcp_connect_config) * 2;
        let vec_tcp_connect = init_tcp_connect::<TBRW, MSG>(&os_epoll, vec_tcp_connect_config);

        Ok(TcpConnectService {
            os_epoll,
            net_msg_cb_fn,
            err_msg_cb_fn,
            vec_tcp_connect,
            phantom: PhantomData,
            vec_shared: vec![0u8; vec_shared_size],
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

    fn read_event(&mut self, sid: u64) {
        //info!("read id:{}", id);
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(sid as usize) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                loop {
                    match tcp_socket.read(&mut self.vec_shared) {
                        ReadResult::Data(msg) => {
                            (self.net_msg_cb_fn)(sid, msg);
                        }
                        ReadResult::BufferIsEmpty => {
                            break;
                        }
                        ReadResult::ReadZeroSize => {
                            warn!("tcp_socket.reader.read :{}", "Read Zero Size");
                            epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                            tcp_connect.set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(
                                &self.os_epoll,
                                &tcp_connect,
                            ));
                            break;
                        }
                        ReadResult::Error(err) => {
                            error!("tcp_socket.reader.read id:{} err:{}", sid, err);
                            epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                            tcp_connect.set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(
                                &self.os_epoll,
                                &tcp_connect,
                            ));
                            break;
                        }
                    }
                }
            }
        } else {
            warn!("read_event tcp_connect_mgmt id no exitis:{}", sid);
        };
    }

    #[inline]
    pub fn write_net_msg(&mut self, sid: u64, msg: MSG) {
        match self.vec_tcp_connect.get_mut(sid as usize) {
            Some(tcp_connect) => {
                let msg_deque_max_len = tcp_connect.get_config().msg_deque_max_len;
                if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                    if tcp_socket.vec_queue_len() > msg_deque_max_len {
                        warn!("sid:{} Msg Queue Is Full", sid);
                        (self.err_msg_cb_fn)(sid, ErrMsg::MsgQueueIsFull);
                        return;
                    }

                    tcp_socket.push_vec_queue(msg);

                    if tcp_socket.vec_queue_len() == 1 {
                        if let Err(err) = write_data(&self.os_epoll, sid, tcp_socket) {
                            warn!("sid:{} write_data  err:{}", sid, err);
                            epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                            tcp_connect.set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(
                                &self.os_epoll,
                                &tcp_connect,
                            ));
                        }
                    }
                }
            }
            None => {
                warn!("write_net_msg socket id no exitis:{}", sid);
                (self.err_msg_cb_fn)(sid, ErrMsg::SocketIdNotExist);
            }
        }
    }

    fn write_event(&mut self, sid: u64) {
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(sid as usize) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                if let Err(err) = write_data(&self.os_epoll, sid, tcp_socket) {
                    warn!("tcp_socket.writer.write sid:{} err:{}", sid, err);
                    epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
                    tcp_connect.set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(
                        &self.os_epoll,
                        &tcp_connect,
                    ));
                }
            }
        } else {
            error!("write_event sid no exit:{}", sid);
        }
    }

    fn error_event(&mut self, sid: u64, err: String) {
        warn!("error_event sid:{} error:{}", sid, err);
        if let Some(tcp_connect) = self.vec_tcp_connect.get_mut(sid as usize) {
            if let Some(tcp_socket) = tcp_connect.get_tcp_socket_opt() {
                epoll_del_fd(&self.os_epoll, sid, tcp_socket.socket.as_raw_fd());
            }
            tcp_connect
                .set_tcp_socket_opt(tcp_reconnect::<TBRW, MSG>(&self.os_epoll, &tcp_connect));
        }
    }
}

fn get_max_socket_read_buffer(vec_tcp_connect_config: &Vec<TcpConnectConfig>) -> usize {
    let mut buffer_size = 0;
    for cfg in vec_tcp_connect_config {
        if cfg.socket_read_buffer > buffer_size {
            buffer_size = cfg.socket_read_buffer;
        }
    }
    buffer_size as usize
}

fn init_tcp_connect<TBRW, MSG>(
    os_epoll: &OSEpoll,
    vec_tcp_connect_config: Vec<TcpConnectConfig>,
) -> Vec<TcpConnect<MSG>>
where
    TBRW: TcpBufRw<MSG> + Default + 'static,
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

fn epoll_del_fd(os_epoll: &OSEpoll, sid: u64, raw_fd: RawFd) {
    if let Err(err) = os_epoll.ctl_del_fd(sid, raw_fd) {
        warn!("os_epoll.ctl_del_fd({}) Error:{}", sid, err);
    }
}

fn write_data<MSG>(
    os_epoll: &OSEpoll,
    sid: u64,
    tcp_socket: &mut TcpSocket<MSG>,
) -> Result<(), String> {
    match tcp_socket.write() {
        WriteResult::Finish => {
            if tcp_socket.epevs == libc::EPOLLIN {
                return Ok(());
            }
            tcp_socket.epevs = libc::EPOLLIN;
            return os_epoll.ctl_mod_fd(sid, tcp_socket.socket.as_raw_fd(), libc::EPOLLIN);
        }
        WriteResult::BufferFull => {
            if tcp_socket.epevs == EPOLL_IN_OUT {
                return Ok(());
            }
            tcp_socket.epevs = EPOLL_IN_OUT;
            return os_epoll.ctl_mod_fd(sid, tcp_socket.socket.as_raw_fd(), EPOLL_IN_OUT);
        }
        WriteResult::Error(err) => return Err(err),
    }
}

/// 断线重连
fn tcp_reconnect<TBRW, MSG>(
    os_epoll: &OSEpoll,
    tcp_connect: &TcpConnect<MSG>,
    //tcp_buf_rw: Box<dyn TcpBufRw<MSG>>,
) -> Option<TcpSocket<MSG>>
where
    TBRW: TcpBufRw<MSG> + Default + 'static,
{
    let ts = time::timestamp();
    if tcp_connect.get_last_reconnect_timestamp()
        + tcp_connect.get_config().reconnect_interval as u64
        > ts
    {
        return None;
    }

    tcp_connect.set_last_reconnect_timestamp(ts);
    match new_tcp_socket::<TBRW, MSG>(os_epoll, tcp_connect.get_sid(), tcp_connect.get_config()) {
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
    sid: u64,
    config: &TcpConnectConfig,
) -> Result<TcpSocket<MSG>, String>
where
    TBRW: TcpBufRw<MSG> + Default + 'static,
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
                    let mut tcp_buf_rw = Box::new(TBRW::default());
                    tcp_buf_rw.set_msg_max_size(config.msg_max_size);
                    return Ok(TcpSocket::new(socket, tcp_buf_rw));
                }
                Err(err) => return Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
