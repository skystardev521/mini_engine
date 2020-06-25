use crate::clients::Clients;
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
            match client.tcp_reader.read(&mut client.stream) {
                Ok(_) => info!("ok"),
                Err(_) => info!("ok"),
            }
        } else {
            error!("client Id:{} Not exists", &id)
        }
    }

    pub fn write(&mut self, id: u64) {
        info!("tcp_event.write({})", id);

        if let Some(client) = self.clients.get_mut_client(id) {
            match client.tcp_writer.write(&mut client.stream) {
                Ok(result) => {
                    /*
                    match result {
                        EnumResult::
                    }
                    */
                    error!("write result :{:?}", result)
                }
                Err(err) => println!("err:{}", err),
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

    pub fn accept(&mut self, socket: TcpStream, addr: SocketAddr) {
        match socket.set_nonblocking(true) {
            Ok(()) => {
                info!("new TcpStream:{}", addr);
                self.clients.new_client(socket, |net_data: Box<MsgData>| {
                    info!("id:{}", &net_data.id)
                });
            }
            Err(err) => {
                info!("new TcpStream:{}", addr);
                info!("set_nonblocking:{}", err);
                match socket.shutdown(Shutdown::Both) {
                    Ok(()) => (),
                    Err(err) => error!("shutdown:{}", err),
                }
            }
        }
    }
}
