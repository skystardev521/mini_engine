use crate::message::MsgEnum;
use crate::message::NetMsg;
use crate::message::SysMsg;
use crate::message::SysMsgId;
use crate::os_epoll::OSEpoll;
use crate::os_socket;
use crate::tcp_listen::TcpListen;
use crate::tcp_listen_config::TcpListenConfig;
use crate::tcp_socket_mgmt::TcpSocketMgmt;
use crate::tcp_socket_reader::ReadResult;
use crate::tcp_socket_writer::WriteResult;
use libc;
use log::{error, info, warn};
use std::io::Error;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

use std::thread;

use crate::tcp_socket::TcpSocket;

const TCP_LISTEN_ID: u64 = 0;
const EPOLL_IN_OUT: i32 = (libc::EPOLLOUT | libc::EPOLLIN) as i32;

pub struct TcpListenService<'a> {
    os_epoll: OSEpoll,
    tcp_listen: TcpListen,
    config: &'a TcpListenConfig,
    tcp_socket_mgmt: TcpSocketMgmt,
    msg_cb: &'a mut dyn Fn(MsgEnum),
    vec_epoll_event: Vec<libc::epoll_event>,
}

impl<'a> Drop for TcpListenService<'a> {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped TcpListenService while unwinding");
        } else {
            error!("dropped TcpListenService while not unwinding");
        }
    }
}

impl<'a> TcpListenService<'a> {
    pub fn new(
        config: &'a TcpListenConfig,
        msg_cb: &'a mut dyn Fn(MsgEnum),
    ) -> Result<Self, String> {
        let os_epoll: OSEpoll = OSEpoll::new()?;

        let tcp_listen = TcpListen::new(&config.bind_socket_addr)?;
        let rawfd = tcp_listen.get_listen().as_raw_fd();
        os_epoll.ctl_add_fd(TCP_LISTEN_ID, rawfd, libc::EPOLLIN)?;

        let tcp_socket_mgmt = TcpSocketMgmt::new(
            TCP_LISTEN_ID,
            config.msg_max_size,
            config.max_tcp_socket,
            config.wait_write_msg_max_num,
        )?;

        Ok(TcpListenService {
            os_epoll,
            config,
            tcp_listen,
            msg_cb,
            tcp_socket_mgmt,
            vec_epoll_event: vec![
                libc::epoll_event { events: 0, u64: 0 };
                config.epoll_max_events as usize
            ],
        })
    }
    pub fn tick(&mut self) {}

    /// 获取连接的 tcp_sokcet 数量
    #[inline]
    pub fn tcp_socket_count(&self) -> u32 {
        self.tcp_socket_mgmt.tcp_socket_count()
    }
    pub fn epoll_event(&mut self, wait_timeout: i32) -> Result<u32, String> {
        match self.os_epoll.wait(wait_timeout, &mut self.vec_epoll_event) {
            Ok(0) => Ok(0),
            Ok(epevs) => {
                for n in 0..epevs as usize {
                    let event = self.vec_epoll_event[n];
                    if event.u64 == TCP_LISTEN_ID {
                        self.accept_event();
                        continue;
                    }

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
                return Ok(epevs);
            }
            Err(err) => Err(err),
        }
    }

    fn read_event(&mut self, sid: u64) {
        info!("read id:{}", sid);
        if let Some(tcp_socket) = self.tcp_socket_mgmt.get_tcp_socket(sid) {
            loop {
                match tcp_socket.reader.read(&mut tcp_socket.socket) {
                    ReadResult::Data(data) => {
                        let msg = NetMsg {
                            sid: sid,
                            data: data,
                        };
                        (self.msg_cb)(MsgEnum::NetMsg(msg));
                    }
                    ReadResult::BufferIsEmpty => {
                        break;
                    }
                    ReadResult::ReadZeroSize => {
                        self.del_tcp_socket(sid, true);
                        warn!("tcp_socket.reader.read :{}", "Read Zero Size");
                        break;
                    }
                    ReadResult::Error(err) => {
                        self.del_tcp_socket(sid, true);
                        error!("tcp_socket.reader.read id:{} err:{}", sid, err);
                        break;
                    }
                }
            }
        } else {
            warn!("read_event tcp_socket_mgmt id no exitis:{}", sid);
        };
    }

    #[inline]
    pub fn write_net_msg(&mut self, net_msg: NetMsg) {
        let msg_max_num = self.tcp_socket_mgmt.get_wait_write_msg_max_num();
        match self.tcp_socket_mgmt.get_tcp_socket(net_msg.sid) {
            Some(tcp_socket) => {
                if tcp_socket.writer.get_msg_data_count() > msg_max_num {
                    self.del_tcp_socket(net_msg.sid, true);
                    warn!("net_msg.id:{} Too much msg_data not send", net_msg.sid);
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
                        self.del_tcp_socket(net_msg.sid, true);
                        warn!("tcp_socket.writer.write err:{}", err);
                    }
                }
            }
            None => {
                self.send_sys_msg(net_msg.sid, SysMsgId::SocketClose);
                warn!("write_net_msg net_msg.id no exitis:{}", net_msg.sid);
            }
        }
    }

    fn write_event(&mut self, sid: u64) {
        if let Some(tcp_socket) = self.tcp_socket_mgmt.get_tcp_socket(sid) {
            if let Err(err) = write_data(&self.os_epoll, sid, tcp_socket) {
                self.del_tcp_socket(sid, true);
                warn!("tcp_socket.writer.write err:{}", err);
            }
        } else {
            warn!("write_event tcp_socket_mgmt id no exitis:{}", sid);
        }
    }

    fn error_event(&mut self, sid: u64, err: String) {
        self.del_tcp_socket(sid, true);
        warn!("os_epoll event sid:{} error:{}", sid, err);
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

        if let Err(err) = socket.set_nodelay(self.config.tcp_nodelay_value) {
            error!("new_socket set_nodelay:{}", err);
            return;
        }

        let raw_fd = socket.as_raw_fd();

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

        match self.tcp_socket_mgmt.add_tcp_socket(socket) {
            Ok(sid) => {
                error!("tcp_socket_mgmt.add_tcp_socket sid:{}", sid);
                match self.os_epoll.ctl_add_fd(sid, raw_fd, libc::EPOLLIN) {
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
    /// is_send_logic:删除后要不要通知业务层
    pub fn del_tcp_socket(&mut self, sid: u64, is_send_logic: bool) {
        match self.tcp_socket_mgmt.del_tcp_socket(sid) {
            Ok(tcp_socket) => {
                let rawfd = tcp_socket.socket.as_raw_fd();
                if let Err(err) = self.os_epoll.ctl_del_fd(sid, rawfd) {
                    warn!("os_epoll.ctl_del_fd({}) Error:{}", sid, err);
                }
                if is_send_logic {
                    self.send_sys_msg(sid, SysMsgId::SocketClose);
                }
            }
            Err(err) => {
                warn!("tcp_socket_mgmt.del_tcp_socket({}) Error:{}", sid, err);
            }
        }
    }

    #[inline]
    fn send_sys_msg(&mut self, sid: u64, smid: SysMsgId) {
        let msg = SysMsg {
            sid: sid,
            smid: smid,
        };
        (self.msg_cb)(MsgEnum::SysMsg(msg));
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
