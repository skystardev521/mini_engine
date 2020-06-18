use crate::entity::NetData;
use crate::tcp_reader::TcpReader;
use crate::tcp_writer::TcpWriter;
use std::collections::HashMap;
use std::net::TcpStream;

///数据包头长度 6 个字节
///(包体字节数 13~32位)+(包Id 1~12位) + 任务Id
const HEAD_SIZE: u32 = 6;

///数据最大字节数
const MSG_MAX_SIZE: u32 = 1024 * 1024;

#[derive(Debug)]
pub enum EnumResult {
    OK,
    MsgSizeTooBig,
    MsgSizeTooSmall,

    ReadZeroSize,
    BufferIsEmpty,
    MsgPackIdError,
}
pub struct Client {
    pub stream: TcpStream,
    pub tcp_reader: Box<TcpReader>,
    pub tcp_writer: Box<TcpWriter>,
}

pub struct Clients {
    max_size: u32,
    pub map: Box<HashMap<u64, Client>>,
}

impl Clients {
    /// max client
    /// max_size: net data max size
    pub fn new(max_client: usize, max_size: u32) -> Result<Self, EnumResult> {
        if max_size < HEAD_SIZE {
            return Err(EnumResult::MsgSizeTooSmall);
        }

        if max_size > MSG_MAX_SIZE {
            return Err(EnumResult::MsgSizeTooBig);
        }

        Ok(Clients {
            max_size: max_size,
            map: Box::new(HashMap::with_capacity(max_client)),
        })
    }
    pub fn get_count(&self) -> usize {
        self.map.len()
    }

    pub fn add_client(
        &mut self,
        id: u64,
        stream: TcpStream,
        net_data_cb: fn(Box<NetData>),
    ) -> Option<Client> {
        if self.map.len() == self.map.capacity() {
            return None;
        }
        self.map
            .insert(id, Client::new(stream, self.max_size, net_data_cb))
    }
}

impl Client {
    pub fn new(stream: TcpStream, msg_max_size: u32, net_data_cb: fn(Box<NetData>)) -> Self {
        Client {
            stream: stream,
            tcp_writer: TcpWriter::new(),
            tcp_reader: TcpReader::new(msg_max_size, net_data_cb),
        }
    }
}
