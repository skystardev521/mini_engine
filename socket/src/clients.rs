use crate::tcp_reader::TcpReader;
use crate::tcp_writer::TcpWriter;
use std::collections::HashMap;
use std::net::TcpStream;

///数据最小字节数
const MSG_MIN_SIZE: u32 = 256;
///数据最大字节数
const MSG_MAX_SIZE: u32 = 1024 * 1024;

pub enum EnumResult {
    OK,
    MsgSizeTooBig,
    MsgSizeTooSmall,
}

pub struct Client {
    stream: TcpStream,
    tcp_reader: Box<TcpReader>,
    tcp_writer: Box<TcpWriter>,
    //socket_addr: SocketAddr,  TcpStream.peer_addr(&self) -> Result<SocketAddr>
}

pub struct Clients {
    msg_max_size: u32,
    hash_map: Box<HashMap<u64, Client>>,
}

impl Clients {
    /// max client
    /// max_size: net data max size
    pub fn new(count: usize, msg_max_size: u32) -> Result<Self, EnumResult> {
        if msg_max_size < MSG_MIN_SIZE {
            return Err(EnumResult::MsgSizeTooSmall);
        }

        if msg_max_size > MSG_MAX_SIZE {
            return Err(EnumResult::MsgSizeTooBig);
        }
        Ok(Clients {
            msg_max_size: msg_max_size,
            hash_map: Box::new(HashMap::with_capacity(count)),
        })
    }
    pub fn get_count(&self) -> usize {
        self.hash_map.len()
    }

    pub fn add_Client(&mut self, id: u64, stream: TcpStream) -> Option<Client> {
        if self.hash_map.len() == self.hash_map.capacity() {
            return None;
        }
        self.hash_map
            .insert(id, Client::new(stream, self.msg_max_size))
    }
}

impl Client {
    /// max_size: net data max size
    /// net_task_cb: new net task call
    pub fn new(stream: TcpStream, msg_max_size: u32) -> Self {
        Client {
            stream: stream,
            tcp_writer: TcpWriter::new(),
            tcp_reader: TcpReader::new(msg_max_size),
        }
    }
}
