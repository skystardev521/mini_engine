use crate::clients::Clients;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;

use crate::entity::NetData;

pub struct TcpEvent<'a> {
    clients: &'a mut Clients,
}

impl<'a> TcpEvent<'a> {
    pub fn new(clients: &'a mut Clients) -> Self {
        TcpEvent { clients: clients }
    }

    pub fn read(&mut self, id: u64) {
        /*
        if let Some(client) = self.clients.map.get_mut(&id) {
            client.read(|net_data: Box<NetData>| println!("id:{}", &net_data.id));
        }else
        {
            println!("client Id:{} Not exists", &id)
        }
        */
        if let Some(client) = self.clients.map.get_mut(&id) {
            match client.tcp_reader.read(&mut client.stream){
                Ok(_)=> println!("ok"),
                Err(_)=>println!("ok"),
            }
        }else
        {
            println!("client Id:{} Not exists", &id)
        }
        
    }

    pub fn write(&mut self, id: u64) {
        if let Some(client) = self.clients.map.get_mut(&id) {
            
            match client.tcp_writer.write(&mut client.stream){
                Ok(er)=> println!("er:{:?}", er),
                Err(err)=>println!("err:{}", err),
            }
        }else
        {
            println!("client Id:{} Not exists", &id)
        }
    }

    pub fn error(&mut self, id: u64) {
        if let Some(client) = self.clients.map.get_mut(&id) {
            match client.tcp_writer.write(&mut client.stream){
                Ok(er)=> println!("er:{:?}", er),
                Err(err)=>println!("err:{}", err),
            }
        }else
        {
            println!("client Id:{} Not exists", &id)
        }
    }

    pub fn accept(&mut self, tcp_socket: TcpStream, addr: SocketAddr) {
        match tcp_socket.set_nonblocking(true) {
            Ok(()) => {
                println!("new TcpStream:{}", addr);

                self.clients.add_client(123456789, tcp_socket,|net_data: Box<NetData>| println!("id:{}", &net_data.id));
            }
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
