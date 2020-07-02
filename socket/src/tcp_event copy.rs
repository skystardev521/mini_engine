use crate::clients::Clients;
use crate::clients::ReadResult;
use crate::clients::WriteResult;
use crate::epoll::Epoll;
use crate::epoll::EpollEvent;
use crate::message::NetMsg;
use log::error;
use std::cell::RefCell;
use std::net::Shutdown;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

pub struct TcpEvent<'a, 'b, 'c> {
    epoll: &'a Epoll,
    clients: &'b mut Clients<'b>,
    //clients: &'b RefCell<Clients<'b>>,
    net_msg_cb: &'c mut dyn FnMut(NetMsg) -> bool,
}

impl<'a, 'b, 'c> TcpEvent<'a, 'b, 'c> {
    pub fn new(
        epoll: &'a Epoll,
        clients: &'b mut Clients<'b>,
        //clients: &'b RefCell<Clients<'b>>,
        net_msg_cb: &'c mut dyn FnMut(NetMsg) -> bool,
    ) -> Self {
        TcpEvent {
            epoll: epoll,
            clients: clients,
            net_msg_cb: net_msg_cb,
        }
    }
}

impl<'a, 'b, 'c> EpollEvent for TcpEvent<'a, 'b, 'c> {
    fn read(&mut self, id: u64) -> bool {
        if let Some(client) = self.clients.borrow_mut().get_client(id) {
            loop {
                match client.tcp_reader.read(&mut client.stream) {
                    ReadResult::Data(msg_data) => {
                        if (self.net_msg_cb)(NetMsg {
                            id: id,
                            data: msg_data,
                        }) == false
                        {
                            return false;
                        }
                    }
                    ReadResult::BufferIsEmpty => {
                        break;
                    }
                    ReadResult::ReadZeroSize => {
                        error!("read({}) ReadZeroSize", id);
                        if let Err(err) = self.clients.borrow_mut().del_client(id) {
                            error!("clients.del_client({}) Error:{}", id, err);
                        }
                        break;
                    }
                    ReadResult::Error(err) => {
                        error!("read({}) error:{}", id, err);
                        if let Err(err) = self.clients.borrow_mut().del_client(id) {
                            error!("clients.del_client({}) Error:{}", id, err);
                        }
                        break;
                    }
                }
            }
        } else {
            error!("client Id:{} Not exists", &id)
        }
        return true;
    }

    fn write(&mut self, id: u64) -> bool {
        if let Some(client) = self.clients.borrow_mut().get_client(id) {
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
                    if let Err(err) = self.clients.borrow_mut().del_client(id) {
                        error!("clients.del_client({}) Error:{}", id, err);
                    }
                }
            }
        } else {
            error!("client Id:{} Not exists", &id)
        }
        return true;
    }

    fn error(&mut self, id: u64, err: String) -> bool {
        error!("error error:{}", err);
        match self.clients.borrow_mut().del_client(id) {
            Ok(()) => (),
            Err(err) => error!("{}", err),
        }
        return true;
    }

    fn accept(&mut self, stream: TcpStream) -> bool {
        match stream.set_nonblocking(true) {
            Ok(()) => match self.clients.borrow_mut().new_client(stream) {
                Ok(()) => return true,
                Err(err) => {
                    error!("new_client:{}", err);
                    return false;
                }
            },
            Err(err) => {
                error!("set_nonblocking:{}", err);
                match stream.shutdown(Shutdown::Both) {
                    Ok(()) => (),
                    Err(err) => error!("shutdown:{}", err),
                }
                return false;
            }
        }
    }
}
