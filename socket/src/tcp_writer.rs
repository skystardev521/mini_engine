use std::collections::LinkedList;
use std::io::prelude::Write;

use std::io::{Error, ErrorKind};
use std::net::TcpStream;

use crate::entity::NetData;
use utils::bytes;

const MAX_ID: u16 = 4096;
///数据包头长度 6 个字节
///(包体字节数 13~32位)+(包Id 1~12位) + NetDataId
const HEAD_SIZE: usize = 6;

pub enum EnumResult {
    OK,
    WriteZeroSize,
}

pub struct TcpWriter {
    id: u16,
    //有任务未完成
    //have_net_data: bool,
    //已发送的字节
    head_pos: usize,
    head_data: [u8; HEAD_SIZE],
    //已发送的字节
    buffer_pos: usize,
    list: LinkedList<Box<NetData>>,
}

impl TcpWriter {
    pub fn new() -> Box<Self> {
        Box::new(TcpWriter {
            id: 0,
            head_pos: 0,
            buffer_pos: 0,
            //have_net_data: false,
            list: LinkedList::new(),
            head_data: [0u8; HEAD_SIZE],
        })
    }

    pub fn get_net_data_count(&self) -> usize {
        self.list.len()
    }

    pub fn add_net_data(&mut self, net_data: Box<NetData>) {
        self.list.push_back(net_data);
    }

    pub fn loop_write(&mut self, stream: &mut TcpStream) -> Result<EnumResult, Error> {
        while let Some(net_data) = self.list.front() {
            if self.head_pos < HEAD_SIZE || self.buffer_pos < net_data.buffer.len() {
                let id = self.id;
                self.head_pos = 0;
                self.buffer_pos = 0;
                self.id = if id == MAX_ID { 0 } else { self.id + 1 };

                let data_size = net_data.buffer.len() as u32;
                let data_size_id = data_size << 12 + id as u32;

                bytes::write_u32(&mut self.head_data[0..], data_size_id);
                bytes::write_u16(&mut self.head_data[4..], net_data.id);
            }

            if self.head_pos < HEAD_SIZE {
                loop {
                    match stream.write(&self.head_data[self.head_pos..]) {
                        Ok(size) => {
                            if 0 == size {
                                //已写满缓冲区 不能再写到缓存区
                                return Ok(EnumResult::WriteZeroSize);
                            }
                            self.head_pos += size;
                            if self.head_pos == HEAD_SIZE {
                                break; //已写完 head_data
                            } else {
                                //已写满缓冲区 不能再写到缓存区
                                return Ok(EnumResult::WriteZeroSize);
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::Interrupted => {
                            continue; //系统中断 再写一次
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            if self.buffer_pos < net_data.buffer.len() {
                loop {
                    match stream.write(&net_data.buffer[self.buffer_pos..]) {
                        Ok(size) => {
                            if 0 == size {
                                //已写满缓冲区 不能再写到缓存区
                                return Ok(EnumResult::WriteZeroSize);
                            }
                            self.buffer_pos += size;
                            if self.buffer_pos == net_data.buffer.len() {
                                //已写完当前buffer所有数据
                                self.list.pop_front();
                                break;
                            } else {
                                //已写满缓冲区 不能再写到缓存区
                                return Ok(EnumResult::WriteZeroSize);
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::Interrupted => {
                            continue; //系统中断 再写一次
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        Ok(EnumResult::OK)
    }
}
