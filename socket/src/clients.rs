use crate::epoll::Epoll;
use crate::message;
use crate::message::MsgData;
use crate::tcp_reader::TcpReader;
use crate::tcp_writer::TcpWriter;
use libc;
use std::collections::HashMap;
//use std::net::Shutdown;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

pub enum NewClientsResult<'a> {
    MsgSizeTooBig,
    ClientNumTooSmall,
    Ok(Clients<'a>),
}

pub enum NewClientResult {
    NewClientSucc,
    MaxClientCount,
    EpollCtlAddFdErr(String),
}

pub enum ReadResult {
    MsgIdError,
    Error(String),
    ReadZeroSize,
    MsgSizeTooBig,
    BufferIsEmpty,
    Data(Box<MsgData>),
}

#[derive(Debug)]
pub enum WriteResult {
    Finish,
    BufferFull,
    Error(String),
}

pub struct Client {
    pub stream: TcpStream,
    pub tcp_reader: Box<TcpReader>,
    pub tcp_writer: Box<TcpWriter>,
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
    pub fn new(max_client: u16, msg_max_size: u32, epoll: &'a Epoll) -> NewClientsResult {
        if max_client < 8 {
            return NewClientsResult::ClientNumTooSmall;
        }

        if msg_max_size > message::MSG_MAX_SIZE {
            return NewClientsResult::MsgSizeTooBig;
        }

        NewClientsResult::Ok(Clients {
            last_id: 0,
            epoll: epoll,
            msg_max_size: msg_max_size,
            hashmap: Box::new(HashMap::with_capacity(max_client as usize)),
        })
    }

    pub fn client_count(&self) -> u32 {
        self.hashmap.len() as u32
    }

    pub fn new_client(&mut self, stream: TcpStream) -> NewClientResult {
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

        self.hashmap
            .insert(self.last_id, Client::new(stream, self.msg_max_size));
        NewClientResult::NewClientSucc
    }

    pub fn del_client(&mut self, id: u64) -> Result<(), String> {
        if let Some(client) = self.hashmap.remove(&id) {
            //client.stream.shutdown(Shutdown::Both);
            match self.epoll.ctl_del_fd(id, client.stream.as_raw_fd()) {
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
    pub fn new(stream: TcpStream, msg_max_size: u32) -> Self {
        Client {
            stream: stream,
            tcp_writer: TcpWriter::new(msg_max_size),
            tcp_reader: TcpReader::new(msg_max_size),
        }
    }
}
