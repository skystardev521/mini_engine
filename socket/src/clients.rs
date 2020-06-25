use crate::epoll::Epoll;
use crate::message::MsgData;
use crate::tcp_reader::TcpReader;
use crate::tcp_writer::TcpWriter;
use libc;
use std::collections::HashMap;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;

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

pub enum NewClientResult {
    NewClientSucc,
    MaxClientCount,
    EpollCtlAddFdErr(String),
}

pub struct Clients<'a> {
    last_id: u64,
    epoll: &'a Epoll,
    msg_max_size: u32,
    hashmap: Box<HashMap<u64, Client>>,
}

impl<'a> Clients<'a> {
    /// max client
    /// max_size: net data max size
    pub fn new(max_client: u32, msg_max_size: u32, epoll: &'a Epoll) -> Result<Self, EnumResult> {
        if msg_max_size > MSG_MAX_SIZE {
            return Err(EnumResult::MsgSizeTooBig);
        }

        Ok(Clients {
            last_id: 0,
            epoll: epoll,
            msg_max_size: msg_max_size,
            hashmap: Box::new(HashMap::with_capacity(max_client as usize)),
        })
    }
    pub fn client_count(&self) -> u32 {
        self.hashmap.len() as u32
    }

    pub fn new_client(
        &mut self,
        stream: TcpStream,
        net_data_cb: fn(Box<MsgData>),
    ) -> NewClientResult {
        if self.hashmap.len() == self.hashmap.capacity() {
            return NewClientResult::MaxClientCount;
        }

        loop {
            self.last_id += 1;
            //tcp_listen id = 0
            if self.last_id == 0 {
                self.last_id = 1;
            }
            if !self.hashmap.contains_key(&self.last_id) {
                break;
            }
        }

        if let Err(err) = self
            .epoll
            .ctl_add_fd(self.last_id, stream.as_raw_fd(), libc::EPOLLIN)
        {
            return NewClientResult::EpollCtlAddFdErr(err);
        }

        self.hashmap.insert(
            self.last_id,
            Client::new(stream, self.msg_max_size, net_data_cb),
        );
        NewClientResult::NewClientSucc
    }

    pub fn del_client(&mut self, id: u64) -> Result<(), String> {
        if let Some(client) = self.hashmap.remove(&id) {
            match self.epoll.ctl_del_fd(id, client.as_raw_fd()) {
                Ok(()) => return Ok(()),
                Err(err) => return Err(format!("{}", err)),
            }
        } else {
            Err(format!("del_client id:{} not exists", id))
        }
    }

    pub fn get_client(&self, id: u64) -> Option<&Client> {
        self.hashmap.get(&id)
    }

    pub fn get_mut_client(&mut self, id: u64) -> Option<&mut Client> {
        self.hashmap.get_mut(&id)
    }
}

impl Client {
    pub fn new(stream: TcpStream, msg_max_size: u32, net_data_cb: fn(Box<MsgData>)) -> Self {
        Client {
            stream: stream,
            tcp_writer: TcpWriter::new(msg_max_size),
            tcp_reader: TcpReader::new(msg_max_size, net_data_cb),
        }
    }

    pub fn as_raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}
