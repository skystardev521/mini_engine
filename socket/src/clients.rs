use crate::entity::NetData;
//use crate::tcp_reader;
//use crate::tcp_reader::TcpReader;
use crate::tcp_writer::TcpWriter;
use std::collections::HashMap;
use std::net::TcpStream;
use std::io::prelude::Read;
use std::io::{Error, ErrorKind};
use std::mem;
use utils::bytes;

///数据最小字节数
const MSG_MIN_SIZE: u32 = 256;
///数据最大字节数
const MSG_MAX_SIZE: u32 = 1024 * 1024;

//包ID最大值
const MAX_ID: u16 = 4096;
///数据包头长度 6 个字节
///(包体字节数 13~32位)+(包Id 1~12位) + 任务Id
const HEAD_SIZE: usize = 6;

#[derive(Debug)]
pub enum EnumResult {
    OK,
    MsgSizeTooBig,
    MsgSizeTooSmall,

    ReadZeroSize,
    //MsgSizeTooBig,
    BufferIsEmpty,
    MsgPackIdError,
}

pub struct TcpReader {
    //包id(0~4096)
    id: u16,
    head_pos: usize,
    body_max_size: usize,
    net_data: Box<NetData>,
    head_data: [u8; HEAD_SIZE],
}


pub struct Client {
    pub stream: TcpStream,
    pub tcp_reader: Box<TcpReader>,
    pub tcp_writer: Box<TcpWriter>,
    //socket_addr: SocketAddr,  TcpStream.peer_addr(&self) -> Result<SocketAddr>
}

pub struct Clients {
    msg_max_size: u32,
    pub map: Box<HashMap<u64, Client>>,
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
            map: Box::new(HashMap::with_capacity(count)),
        })
    }
    pub fn get_count(&self) -> usize {
        self.map.len()
    }

    pub fn add_client(&mut self, id: u64, stream: TcpStream) -> Option<Client> {
        if self.map.len() == self.map.capacity() {
            return None;
        }
        self.map.insert(id, Client::new(stream, self.msg_max_size))
    }
}

impl Client {
    /// max_size: net data max size
    pub fn new(stream: TcpStream, msg_max_size: u32) -> Self {
        Client {
            stream: stream,
            tcp_writer: TcpWriter::new(),
            tcp_reader: Client::new_tcp_reader(msg_max_size),//TcpReader::new(msg_max_size),
        }
    }

    pub(crate) fn new_tcp_reader(msg_max_size: u32) -> Box<TcpReader> {
        Box::new(TcpReader {
            id: 0,
            head_pos: 0,
            head_data: [0u8; HEAD_SIZE],
            body_max_size: msg_max_size as usize - HEAD_SIZE,
            net_data: Box::new(NetData {
                id: 0,
                buffer: vec![0u8; 0],
            }),
        })
    }

    pub fn read(&mut self, net_data_cb: fn(Box<NetData>)) -> Result<EnumResult, Error> {
        loop {
            if self.tcp_reader.head_pos != HEAD_SIZE {
                loop {
                    let head_pos = self.tcp_reader.head_data.len();
                    match self.stream.read(&mut self.tcp_reader.head_data[head_pos..]) {
                        Ok(0) => {
                            return Ok(EnumResult::ReadZeroSize);
                        }
                        Ok(_size) => {
                            //读取到的字节数
                            if self.tcp_reader.head_data.len() == HEAD_SIZE {
                                let data = bytes::read_u32(&self.tcp_reader.head_data);
                                let pack_id = (data << 20 >> 20) as u16;

                                if pack_id != self.tcp_reader.id {
                                    return Ok(EnumResult::MsgPackIdError);
                                }
                                if self.tcp_reader.id == MAX_ID {
                                    self.tcp_reader.id = 0
                                } else {
                                    self.tcp_reader.id += 1;
                                };

                                let buffer_size = (data >> 12) as usize;
                                if buffer_size > self.tcp_reader.body_max_size {
                                    return Ok(EnumResult::MsgSizeTooBig);
                                }

                                let id = bytes::read_u16(&self.tcp_reader.head_data[4..]);
                                if buffer_size == 0 {
                                    //读完一个包
                                    self.tcp_reader.head_pos = 0;
                                    net_data_cb(Box::new(NetData {
                                        id: id,
                                        buffer: vec![],
                                    }));
                                    continue;
                                } else {
                                    self.tcp_reader.net_data = Box::new(NetData {
                                        id: id,
                                        buffer: vec![0u8; buffer_size],
                                    });
                                    break; //读完包头数据
                                }
                            }
                            //缓冲区已读完 包头数据 还没有读完
                            return Ok(EnumResult::BufferIsEmpty);
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            println!("ErrorKind::WouldBlock");
                            //缓冲区已读完 包头数据 还没有读完
                            return Ok(EnumResult::BufferIsEmpty);
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                            println!("ErrorKind::Interrupted");
                            continue; ////系统中断 再read一次
                        }
                        Err(e) => return Err(e),
                    }
                }
            }

            loop {
                let buffer_pos = self.tcp_reader.net_data.buffer.len();
                match self.stream.read(&mut self.tcp_reader.net_data.buffer[buffer_pos..]) {
                    Ok(0) => {
                        println!("ErrorKind::Interrupted");
                        println!("stream.read result 0");
                        return Ok(EnumResult::ReadZeroSize);
                    }
                    Ok(_size) => {
                        //读取到的字节数
                        if self.tcp_reader.net_data.buffer.len() == self.tcp_reader.net_data.buffer.capacity() {
                            //读完一个包
                            let tmp_net_data = Box::new(NetData {
                                id: 0,
                                buffer: vec![],
                            });
                            net_data_cb(mem::replace(&mut self.tcp_reader.net_data, tmp_net_data));
                            break;
                        } else {
                            //缓冲区已读完 包头数据 还没有读完
                            return Ok(EnumResult::BufferIsEmpty);
                        }
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        println!("ErrorKind::WouldBlock");
                        //缓冲区已读完 包头数据 还没有读完
                        return Ok(EnumResult::BufferIsEmpty);
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                        println!("ErrorKind::Interrupted");
                        continue; ////系统中断 再read一次
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
}
