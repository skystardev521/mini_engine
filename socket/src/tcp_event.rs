use crate::clients::Clients;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;

pub struct TcpEvent<'a> {
    clients: &'a mut Clients,
}

impl<'a> TcpEvent<'a> {
    pub fn new(clients: &'a mut Clients) -> Self {
        TcpEvent { clients: clients }
    }

    pub fn recv(&self, id: u64) {}

    pub fn send(&self, id: u64) {
        //clients.hash_map.
    }

    pub fn error(&self, id: u64) {}

    pub fn accept(&self, tcp_socket: TcpStream, addr: SocketAddr) {
        match tcp_socket.set_nonblocking(true) {
            Ok(()) => println!("new TcpStream:{}", addr),
            Err(err) => {
                println!("new TcpStream:{}", addr);
                println!("set_nonblocking:{}", err);
                match tcp_socket.shutdown(Shutdown::Both) {
                Ok(()) => (),
                Err(err) => println!("shutdown:{}", err),
                }
            }
        }
    }
}
