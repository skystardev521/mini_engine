use std::collections::VecDeque;
use std::io::prelude::Write;

use std::io::ErrorKind;
use std::net::TcpStream;

use crate::message;
use crate::message::MsgData;
use utils::bytes;

#[derive(Debug)]
pub enum EnumResult {
    OK,
    WriteZeroSize,
    MsgSizeTooBig,
}

pub struct TcpWriter {
    id: u16,
    headpos: usize,
    datapos: usize,
    maxsize: usize,
    headdata: [u8; message::MSG_HEAD_SIZE],
    deques: VecDeque<Box<MsgData>>,
    is_write_finish_current_data: bool,
}

impl TcpWriter {
    pub fn new(maxsize: u32) -> Box<Self> {
        Box::new(TcpWriter {
            id: 0,
            headpos: 0,
            datapos: 0,
            deques: VecDeque::new(),
            headdata: [0u8; message::MSG_HEAD_SIZE],
            is_write_finish_current_data: true,
            maxsize: if maxsize > message::MSG_MAX_SIZE {
                message::MSG_MAX_SIZE
            } else {
                maxsize
            } as usize,
        })
    }

    pub fn get_msgdata_count(&self) -> u32 {
        self.deques.len() as u32
    }

    pub fn add_msgdata(&mut self, msgdata: Box<MsgData>) -> Result<(), EnumResult> {
        if msgdata.data.len() > self.maxsize {
            Err(EnumResult::MsgSizeTooBig)
        } else {
            Ok(self.deques.push_back(msgdata))
        }
    }

    #[inline]
    fn id_increment(&self) -> u16 {
        if self.id > message::MSG_MAX_ID {
            0
        } else {
            self.id + 1
        }
    }

    pub fn write(&mut self, stream: &mut TcpStream) -> Result<EnumResult, String> {
        'go_while: while let Some(msgdata) = self.deques.front() {
            //create head data
            if self.is_write_finish_current_data {
                self.is_write_finish_current_data = false;
                self.id = self.id_increment();
                let datasize = msgdata.data.len() as u32;
                let datasizeid = datasize << 12 + self.id as u32;
                bytes::write_u32(&mut self.headdata[0..], datasizeid);
                bytes::write_u16(&mut self.headdata[4..], msgdata.id);
            }

            // write head data
            if self.headpos < message::MSG_HEAD_SIZE {
                loop {
                    match stream.write(&self.headdata[self.headpos..]) {
                        Ok(0) => {
                            //已写满缓冲区 不能再写到缓存区
                            return Ok(EnumResult::WriteZeroSize);
                        }
                        Ok(size) => {
                            self.headpos += size;
                            if self.headpos == message::MSG_HEAD_SIZE {
                                //已写完 headdata
                                if 0 == msgdata.data.len() {
                                    //当前buffer没有数据
                                    self.deques.pop_front();
                                    self.is_write_finish_current_data = true;
                                }
                                break 'go_while;
                            } else {
                                //已写满缓冲区 不能再写到缓存区
                                return Ok(EnumResult::WriteZeroSize);
                            }
                        }
                        Err(err) if err.kind() == ErrorKind::Interrupted => {
                            continue; //系统中断 再写一次
                        }
                        Err(err) => return Err(format!("{}", err)),
                    }
                }
            }
            //write buffer data
            if self.datapos < msgdata.data.len() {
                loop {
                    match stream.write(&msgdata.data[self.datapos..]) {
                        Ok(0) => {
                            //已写满缓冲区 不能再写到缓存区
                            return Ok(EnumResult::WriteZeroSize);
                        }
                        Ok(size) => {
                            self.datapos += size;
                            if self.datapos == msgdata.data.len() {
                                //已写完当前buffer所有数据
                                self.deques.pop_front();
                                self.is_write_finish_current_data = true;
                                break 'go_while; //已写完
                            } else {
                                //已写满缓冲区 不能再写到缓存区
                                return Ok(EnumResult::WriteZeroSize);
                            }
                        }
                        Err(err) if err.kind() == ErrorKind::Interrupted => {
                            continue; //系统中断 再写一次
                        }
                        Err(err) => return Err(format!("{}", err)),
                    }
                }
            }
        }
        Ok(EnumResult::OK)
    }
}
