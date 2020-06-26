use crate::clients::Clients;
use crate::clients::ReadResult;
use crate::clients::WriteResult;
use crate::epoll::Epoll;
use crate::message::MsgData;
use log::{error, info};
use std::io::Error;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;

pub struct EpEvent<'a> {
    epoll: &'a Epoll,
    clients: &'a mut Clients<'a>,
}

impl<'a> EpEvent<'a> {
    pub fn new(epoll: &'a Epoll, clients: &'a mut Clients<'a>) -> Self {
        EpEvent {
            epoll: epoll,
            clients: clients,
        }
    }

    pub fn read(&mut self, id: u64) {
        info!("tcp_event.read({})", id);

        if let Some(client) = self.clients.get_mut_client(id) {
            loop {
                match client.tcp_reader.read(&mut client.stream) {
                    ReadResult::Data(msgdata) => {
                        info!(
                            "id:{} data:{:?}",
                            &msgdata.id,
                            String::from_utf8_lossy(&msgdata.data).to_string()
                        );
                    }
                    ReadResult::BufferIsEmpty => {
                        error!("read({}) BufferIsEmpty", id);
                        break;
                    }
                    ReadResult::ReadZeroSize => {
                        error!("read({}) ReadZeroSize", id);
                        if let Err(err) = self.clients.del_client(id) {
                            error!("clients.del_client({}) Error:{}", id, err);
                        }
                        break;
                    }
                    ReadResult::MsgSizeTooBig => {
                        error!("read({}) MsgSizeTooBig", id);
                        if let Err(err) = self.clients.del_client(id) {
                            error!("clients.del_client({}) Error:{}", id, err);
                        }
                        break;
                    }
                    ReadResult::MsgIdError => {
                        error!("read({}) MsgPackIdError", id);
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
        } else {
            error!("client Id:{} Not exists", &id)
        }
    }

    pub fn write(&mut self, id: u64) {
        info!("tcp_event.write({})", id);
        if let Some(client) = self.clients.get_mut_client(id) {
            match client.tcp_writer.write(&mut client.stream) {
                WriteResult::Finish => info!("write result:{}", "Finish"),
                WriteResult::BufferFull => error!("write result:{}", "BufferFull"),
                WriteResult::Error(err) => error!("write result error:{}", err),
            }
        } else {
            error!("client Id:{} Not exists", &id)
        }
    }

    pub fn error(&mut self, id: u64, err: Error) {
        info!("error error:{}", err);

        match self.clients.del_client(id) {
            Ok(()) => (),
            Err(err) => error!("{}", err),
        }
    }

    pub fn accept(&mut self, stream: TcpStream, addr: SocketAddr) {
        match stream.set_nonblocking(true) {
            Ok(()) => {
                info!("new TcpStream:{}", addr);
                self.clients.new_client(stream);
            }
            Err(err) => {
                info!("new TcpStream:{}", addr);
                info!("set_nonblocking:{}", err);
                match stream.shutdown(Shutdown::Both) {
                    Ok(()) => (),
                    Err(err) => error!("shutdown:{}", err),
                }
            }
        }
    }
}
