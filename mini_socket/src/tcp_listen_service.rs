use crate::exc_kind::ExcKind;
use crate::os_epoll::OSEpoll;
use crate::os_socket;
use crate::tcp_listen::TcpListen;
use crate::tcp_listen_config::TcpListenConfig;
use crate::tcp_socket_mgmt::TcpSocketMgmt;
use crate::tcp_socket_rw::ReadResult;
use crate::tcp_socket_rw::TcpSocketRw;
use crate::tcp_socket_rw::WriteResult;

use libc;
use log::{error, info, warn};
use std::io::Error;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

use std::thread;

use crate::tcp_socket::TcpSocket;

const TCP_LISTEN_ID: u32 = 0;
const EPOLL_IN_OUT: i32 = (libc::EPOLLOUT | libc::EPOLLIN) as i32;

pub struct TcpListenService<'a, TBRW, MSG> {
    os_epoll: OSEpoll,
    vec_share: Vec<u8>,
    tcp_listen: TcpListen,
    phantom: PhantomData<TBRW>,
    config: &'a TcpListenConfig,
    tcp_socket_mgmt: TcpSocketMgmt<MSG>,
    vec_epoll_event: Vec<libc::epoll_event>,
    net_msg_cb_fn: &'a mut dyn Fn(u32, Vec<MSG>),
    exc_msg_cb_fn: &'a mut dyn Fn(u32, ExcKind),
}

impl<'a, TBRW, MSG> Drop for TcpListenService<'a, TBRW, MSG> {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped TcpListenService while unwinding");
        } else {
            error!("dropped TcpListenService while not unwinding");
        }
    }
}

impl<'a, TBRW, MSG> TcpListenService<'a, TBRW, MSG>
where
    TBRW: TcpSocketRw<MSG> + Default + 'static,
{
    pub fn new(
        config: &'a TcpListenConfig,
        net_msg_cb_fn: &'a mut dyn Fn(u32, Vec<MSG>),
        exc_msg_cb_fn: &'a mut dyn Fn(u32, ExcKind),
    ) -> Result<Self, String> {
        let os_epoll: OSEpoll = OSEpoll::new()?;

        let tcp_listen = TcpListen::new(&config.bind_socket_addr)?;
        let rawfd = tcp_listen.get_listen().as_raw_fd();
        os_epoll.ctl_add_fd(TCP_LISTEN_ID, rawfd, libc::EPOLLIN)?;

        let tcp_socket_mgmt = TcpSocketMgmt::new(
            TCP_LISTEN_ID,
            config.max_tcp_socket,
            config.msg_deque_max_len as usize,
        );

        let vec_share_size = config.socket_read_buffer as usize * 3;

        Ok(TcpListenService {
            os_epoll,
            config,
            tcp_listen,
            net_msg_cb_fn,
            exc_msg_cb_fn,
            tcp_socket_mgmt,
            phantom: PhantomData,
            vec_share: vec![0u8; vec_share_size],
            vec_epoll_event: vec![
                libc::epoll_event { events: 0, u64: 0 };
                config.epoll_max_events as usize
            ],
        })
    }

    fn write_data(
        cid: u32,
        os_epoll: &OSEpoll,
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
                return os_epoll.ctl_mod_fd(cid , tcp_socket.socket.as_raw_fd(), EPOLL_IN_OUT);
            }
            WriteResult::Error(err) => return Err(err),
        }
    }

    pub fn tick(&mut self) {}

    /// 获取连接的 tcp_sokcet 数量
    #[inline]
    pub fn tcp_socket_count(&self) -> u32 {
        self.tcp_socket_mgmt.tcp_socket_count()
    }

    pub fn epoll_event(&mut self, wait_timeout: i32) -> Result<u32, String> {
        // todo 根据测试代码 死循环向同一条连接中发数据 wait 200多毫秒才会触发一次事件
        match self.os_epoll.wait(wait_timeout, &mut self.vec_epoll_event) {
            Ok(0) => Ok(0),
            Ok(epevs) => {
                for n in 0..epevs as usize {
                    let event = self.vec_epoll_event[n];
                    if event.u64 as u32 == TCP_LISTEN_ID {
                        self.accept_event();
                        continue;
                    }
                    if (event.events & libc::EPOLLIN as u32) != 0 {
                        self.read_event(event.u64 as u32);
                    }
                    if (event.events & libc::EPOLLOUT as u32) != 0 {
                        self.write_event(event.u64 as u32);
                    }
                    if (event.events & libc::EPOLLERR as u32) != 0 {
                        self.error_event(event.u64 as u32, Error::last_os_error().to_string());
                    }
                }
                return Ok(epevs);
            }
            Err(err) => Err(err),
        }
    }

    fn read_event(&mut self, cid: u32) {
        //info!("read id:{}", cid);
        if let Some(tcp_socket) = self.tcp_socket_mgmt.get_tcp_socket(cid) {
            match tcp_socket.read(&mut self.vec_share) {
                ReadResult::Data(vec_msg) => {
                    (self.net_msg_cb_fn)(cid, vec_msg);
                }
                ReadResult::Error(vec_msg, err) => {
                    self.del_tcp_socket(cid);
                    (self.net_msg_cb_fn)(cid, vec_msg);
                    (self.exc_msg_cb_fn)(cid, ExcKind::SocketClose);
                    error!("tcp_socket.read id:{} err:{}", cid, err);
                }
            }
        } else {
            warn!("read_event tcp_socket_mgmt id no exitis:{}", cid);
        };
    }

    #[inline]
    pub fn write_net_msg(&mut self, cid: u32, msg: MSG) {
        let msg_deque_max_len = self.tcp_socket_mgmt.get_msg_deque_max_len();
        match self.tcp_socket_mgmt.get_tcp_socket(cid) {
            Some(tcp_socket) => {
                if tcp_socket.vec_queue_len() > msg_deque_max_len {
                    info!("cid:{} Msg Queue Is Full", cid);
                    (self.exc_msg_cb_fn)(cid, ExcKind::MsgQueueIsFull);
                    return;
                }
                tcp_socket.push_vec_queue(msg);

                if tcp_socket.vec_queue_len() == 1 {
                    if let Err(err) = Self::write_data(cid, &self.os_epoll,tcp_socket) {
                        self.del_tcp_socket(cid);
                        info!("cid:{} write_data  err:{}", cid, err);
                        (self.exc_msg_cb_fn)(cid, ExcKind::SocketClose);
                    }
                }
            }
            None => {
                info!("write_net_msg socket id:{} no exitis", cid);
                (self.exc_msg_cb_fn)(cid, ExcKind::SocketIdNotExist);
            }
        }
    }

    fn write_event(&mut self, cid: u32) {
        if let Some(tcp_socket) = self.tcp_socket_mgmt.get_tcp_socket(cid) {
            if let Err(err) = Self::write_data(cid, &self.os_epoll, tcp_socket) {
                self.del_tcp_socket(cid);
                warn!("write_event cid:{} err:{}", cid, err);
                (self.exc_msg_cb_fn)(cid, ExcKind::SocketClose);
            }
        } else {
            error!("write_event cid:{} no exist", cid);
        }
    }

    fn error_event(&mut self, cid: u32, err: String) {
        self.del_tcp_socket(cid);
        error!("error_event cid:{} error:{}", cid, err);
        (self.exc_msg_cb_fn)(cid, ExcKind::SocketClose);
    }

    fn accept_event(&mut self) {
        loop {
            match self.tcp_listen.get_listen().accept() {
                Ok((socket, addr)) => {
                    self.new_socket(socket);
                    info!("tcp listen serrver new_socket:{}", addr)
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(ref err) => error!("tcp listen serrver accept() error:{}", err),
            }
        }
    }
    fn new_socket(&mut self, socket: TcpStream) {
        if let Err(err) = socket.set_nonblocking(true) {
            error!("new_socket set_nonblocking:{}", err);
            return;
        }

        let raw_fd = socket.as_raw_fd();

        /*
        if let Err(err) = socket.set_nodelay(self.config.tcp_nodelay) {
            error!("new_socket set_nodelay:{}", err);
            return;
        }
        */

        if let Err(err) = os_socket::setsockopt(
            raw_fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            self.config.socket_read_buffer,
        ) {
            error!("new_socket setsockopt SO_RCVBUF :{}", err);
            return;
        }

        if let Err(err) = os_socket::setsockopt(
            raw_fd,
            libc::SOL_SOCKET,
            libc::SO_SNDBUF,
            self.config.socket_write_buffer,
        ) {
            error!("new_socket setsockopt SO_SNDBUF :{}", err);
            return;
        }
        

        match self.tcp_socket_mgmt.add_tcp_socket::<TBRW>(socket) {
            Ok(cid) => {
                info!("tcp_socket_mgmt.add_tcp_socket cid:{}", cid);
                match self.os_epoll.ctl_add_fd(cid, raw_fd, libc::EPOLLIN) {
                    Ok(()) => (),
                    Err(err) => {
                        error!("os_epoll ctl_add_fd error:{}", err);
                    }
                };
            }
            Err(err) => {
                error!("new_socket:{}", err);
            }
        }
    }
    pub fn del_tcp_socket(&mut self, cid: u32) {
        match self.tcp_socket_mgmt.del_tcp_socket(cid) {
            Ok(tcp_socket) => {
                let rawfd = tcp_socket.socket.as_raw_fd();
                if let Err(err) = self.os_epoll.ctl_del_fd(cid, rawfd) {
                    warn!("os_epoll.ctl_del_fd({}) Error:{}", cid, err);
                }
            }
            Err(err) => {
                warn!("tcp_socket_mgmt.del_tcp_socket({}) Error:{}", cid, err);
            }
        }
    }
}
