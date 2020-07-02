use crate::clients::Client;
use crate::clients::ReadResult;
use crate::clients::WriteResult;
use crate::epoll::Epoll;
use crate::epoll::EpollEvent;
use crate::message::NetMsg;
use log::error;
use std::io::Error;
use std::io::ErrorKind;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

pub struct TcpEvent<'a, 'b> {
    epoll: &'a Epoll,
    net_msg_cb: &'b mut dyn Fn(NetMsg),
}

impl<'a, 'b> TcpEvent<'a, 'b> {
    pub fn new(epoll: &'a Epoll, net_msg_cb: &'b mut dyn Fn(NetMsg)) -> Self {
        TcpEvent {
            epoll: epoll,
            net_msg_cb: net_msg_cb,
        }
    }
}

impl<'a, 'b> EpollEvent for TcpEvent<'a, 'b> {
    fn read(&mut self, id: u64, client: &mut Client) {
        //if let Some(client) = self.clients.get_mut_client(id) {
        loop {
            match client.tcp_reader.read(&mut client.stream) {
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
                    error!("read({}) ReadZeroSize", id);
                    if let Err(err) = self.clients.del_client(id) {
                        error!("clients.del_client({}) Error:{}", id, err);
                    }
                    break;
                }
                ReadResult::Error(err) => {
                    error!("read({}) error:{}", id, err);
                    if let Err(err) = self.clients.del_client(id) {
                        error!("clients.del_client({}) Error:{}", id, err);
                    }
                    break;
                }
            }
        }

        /*
        else {
            error!("client Id:{} Not exists", &id)
        }
        */
    }

    fn write(&mut self, id: u64, client: &mut Client) {
        //if let Some(client) = self.clients.get_mut_client(id) {
        match client.tcp_writer.write(&mut client.stream) {
            WriteResult::Finish => (),
            WriteResult::BufferFull => {
                if let Err(err) = self.epoll.ctl_mod_fd(
                    id,
                    client.stream.as_raw_fd(),
                    (libc::EPOLLOUT | libc::EPOLLIN) as i32,
                ) {
                    error!("write ctl_mod_fd err:{}", err);
                }
            }
            WriteResult::Error(err) => {
                error!("write result error:{}", err);
                if let Err(err) = self.clients.del_client(id) {
                    error!("clients.del_client({}) Error:{}", id, err);
                }
            }
        }
        /*
        } else {
            error!("client Id:{} Not exists", &id)
        }
        */
    }

    fn error(&mut self, id: u64, err: String) {
        error!("error error:{}", err);
        match self.clients.del_client(id) {
            Ok(()) => (),
            Err(err) => error!("{}", err),
        }
    }

    fn accept(&mut self, stream: TcpStream) {
        match stream.set_nonblocking(true) {
            Ok(()) => (),
            Err(err) => {
                error!("set_nonblocking:{}", err);
                match stream.shutdown(Shutdown::Both) {
                    Ok(()) => (),
                    Err(err) => error!("shutdown:{}", err),
                }
            }
        }
    }

    /*
    fn accept1(&mut self, new_stream: &dyn Fn() -> Result<(TcpStream, SocketAddr), Error>) {
        loop {
            match new_stream() {
                Ok((stream, _addr)) => {
                    if let Err(err) = stream.set_nonblocking(true) {
                        error!("set_nonblocking:{}", err);
                        if let Err(err) = stream.shutdown(Shutdown::Both) {
                            error!("stream.shutdown:{}", err);
                        }
                        break;
                    }

                    if let Err(err) = self.clients.new_client(stream) {
                        error!("new_client:{}", err);
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(err) => {
                    error!("stream.shutdown:{}", err);
                    break;
                }
            }
        }
    }
    */
}
