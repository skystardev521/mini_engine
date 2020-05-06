use crate::epoll::Epoll;
use std::collections::HashMap;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::rc::Rc;

use crate::entity{
    NetData,
};

use crate::tcp_reader::TcpReader;
use crate::tcp_writer::TcpWriter;

pub struct Client {
    stream: TcpStream,
    tcp_reader: Tcp_reader,
    tcp_writer: TcpWriter,
    //socket_addr: SocketAddr,  TcpStream.peer_addr(&self) -> Result<SocketAddr>
}

pub struct Clients {
    max_size: u32, 
    hash_map: Box<HashMap<u64, Client>>,
}

impl Clients {
    /// max client
    /// max_size: net data max size
    pub fn new(count: usize, max_size: u32) -> Self {
        Clients {
            max_size: max_size,
            hash_map: Box::new(HashMap::with_capacity(count)),
        }
    }
    pub fn get_count(&self) -> usize {
        self.hash_map.len()
    }

    pub fn add_Client(
        &mut self,
        id: u64,
        stream: TcpStream,
    ) -> Option<Client> {
        if self.Clients.len() == self.Clients.capacity() {
            return None;
        }
        self.Clients.insert(id, Client::new(stream, max_size))
    }
}

impl Client {

    /// max_size: net data max size
    /// net_task_cb: new net task call
    fn new(stream: TcpStream, max_size: u32, net_task_cb: fn(NetTask)) -> Self {
        Client {
            stream: stream,
            tcp_writer: TcpWriter::new(),
            tcp_reader: TcpReader::new(buffer_size, max_size, net_task_cb),
        }
    }
}
