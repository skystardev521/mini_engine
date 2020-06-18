use std::collections::VecDeque;
use std::io::prelude::Write;

use std::io::{Error, ErrorKind};
use std::net::TcpStream;

use crate::entity::NetData;
use utils::bytes;

//数据id
const MSG_MAX_ID: u16 = 4096;
///数据包头长度 6 个字节
///(包体字节数 13~32位)+(包Id 1~12位) + 协议id(16位)
const HEAD_SIZE: usize = 6;

#[derive(Debug)]
pub enum EnumResult {
    OK,
    WriteZeroSize,
    MsgSizeTooBig,
}

pub struct TcpWriter {
    id: u16,
    max_size: usize,
    head_pos: usize,
    head_data: [u8; HEAD_SIZE],
    buffer_pos: usize,
    list: VecDeque<Box<NetData>>,
    is_write_finish_current_data: bool,
}

impl TcpWriter {
    pub fn new() -> Box<Self> {
        Box::new(TcpWriter {
            id: 0,
            max_size: 0,
            head_pos: 0,
            buffer_pos: 0,
            list: VecDeque::new(),
            head_data: [0u8; HEAD_SIZE],
            is_write_finish_current_data: true,
        })
    }

    pub fn get_net_data_count(&self) -> usize {
        self.list.len()
    }

    pub fn add_net_data(&mut self, net_data: Box<NetData>) -> Result<(), EnumResult> {
        if net_data.buffer.len() > self.max_size {
            Err(EnumResult::MsgSizeTooBig)
        } else {
            Ok(self.list.push_back(net_data))
        }
    }

    pub fn write(&mut self, stream: &mut TcpStream) -> Result<EnumResult, Error> {
        'go_while: while let Some(net_data) = self.list.front() {
            //create head data
            if self.is_write_finish_current_data {
                self.is_write_finish_current_data = false;

                self.head_pos = 0;
                self.buffer_pos = 0;
                if self.id == MSG_MAX_ID {
                    self.id = 0
                } else {
                    self.id += 1;
                };

                let data_size = net_data.buffer.len() as u32;
                let data_size_id = data_size << 12 + self.id as u32;

                bytes::write_u32(&mut self.head_data[0..], data_size_id);
                bytes::write_u16(&mut self.head_data[4..], net_data.id);
            }

            // write head data
            if self.head_pos < HEAD_SIZE {
                loop {
                    match stream.write(&self.head_data[self.head_pos..]) {
                        Ok(0) => {
                            //已写满缓冲区 不能再写到缓存区
                            return Ok(EnumResult::WriteZeroSize);
                        }
                        Ok(size) => {
                            self.head_pos += size;
                            if self.head_pos == HEAD_SIZE {
                                //已写完 head_data
                                if 0 == net_data.buffer.len() {
                                    //当前buffer没有数据
                                    self.list.pop_front();
                                    self.is_write_finish_current_data = true;
                                }
                                break 'go_while;
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
            //write buffer data
            if self.buffer_pos < net_data.buffer.len() {
                loop {
                    match stream.write(&net_data.buffer[self.buffer_pos..]) {
                        Ok(0) => {
                            //已写满缓冲区 不能再写到缓存区
                            return Ok(EnumResult::WriteZeroSize);
                        }
                        Ok(size) => {
                            self.buffer_pos += size;
                            if self.buffer_pos == net_data.buffer.len() {
                                //已写完当前buffer所有数据
                                self.list.pop_front();
                                self.is_write_finish_current_data = true;
                                break 'go_while; //已写完
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
