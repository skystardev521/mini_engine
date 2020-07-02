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

pub enum ReadResult {
    Error(String),
    ReadZeroSize,
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
    max_size: u32,
    epoll: &'a Epoll,
    hash_map: HashMap<u64, Client>,
}

impl<'a> Clients<'a> {
    /// max client
    /// max_size: data max size
    pub fn new(max_client: u16, max_size: u32, epoll: &'a Epoll) -> Result<Self, String> {
        if max_client < 8 {
            return Err("ClientNumTooSmall".into());
        }

        if max_size > message::MSG_MAX_SIZE {
            return Err("MsgSizeTooBig".into());
        }

        Ok(Clients {
            epoll: epoll,
            max_size: max_size,
            //epoll_event: epoll_event,
            last_id: 0,
            hash_map: HashMap::with_capacity(max_client as usize),
        })
    }

    pub fn client_count(&self) -> u32 {
        self.hash_map.len() as u32
    }

    pub fn new_client(&mut self, stream: TcpStream) -> Result<(), String> {
        if self.hash_map.len() == self.hash_map.capacity() {
            return Err("MaxClientCount".into());
        }
        loop {
            self.last_id += 1;
            if self.last_id == 0 {
                self.last_id = 1;
            }
            if self.hash_map.contains_key(&self.last_id) == false {
                break;
            }
        }

        if let Err(err) = self
            .epoll
            .ctl_add_fd(self.last_id, stream.as_raw_fd(), libc::EPOLLIN)
        {
            return Err(err);
        }

        self.hash_map
            .insert(self.last_id, Client::new(stream, self.max_size));
        Ok(())
    }

    pub fn del_client(&mut self, id: u64) -> Result<(), String> {
        if let Some(client) = self.hash_map.remove(&id) {
            //client.stream.shutdown(Shutdown::Both);
            match self.epoll.ctl_del_fd(id, client.stream.as_raw_fd()) {
                Ok(()) => return Ok(()),
                Err(err) => return Err(format!("{}", err)),
            }
        } else {
            Err(format!("del_client id:{} not exists", id))
        }
    }

    pub fn get_client(&mut self, id: u64) -> Option<&mut Client> {
        self.hash_map.get_mut(&id)
    }
}

impl Client {
    pub fn new(stream: TcpStream, max_size: u32) -> Self {
        Client {
            stream: stream,
            tcp_writer: TcpWriter::new(max_size),
            tcp_reader: TcpReader::new(max_size),
        }
    }
}
