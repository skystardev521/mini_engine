/*
use crate::message;
use mini_utils::bytes;
use std::collections::VecDeque;
use std::io::prelude::Write;
use std::io::ErrorKind;
use std::net::TcpStream;

#[derive(Debug)]
pub enum WriteResult {
    Finish,
    BufferFull,
    Error(String),
}

pub struct TcpSocketWriter {
    next_mid: u16,
    head_pos: usize,
    data_pos: usize,
    max_size: usize,
    vec_deque: VecDeque<Vec<u8>>,
    is_write_finish_current_data: bool,
    head_data: [u8; message::MSG_HEAD_SIZE],
}

impl TcpSocketWriter {
    pub fn new(msg_max_size: u32) -> Box<Self> {
        let mut max_size = msg_max_size;
        if max_size > message::MSG_MAX_SIZE {
            max_size = message::MSG_MAX_SIZE
        }
        Box::new(TcpSocketWriter {
            next_mid: 0,
            head_pos: 0,
            data_pos: 0,
            max_size: max_size as usize,
            vec_deque: VecDeque::new(),
            is_write_finish_current_data: true,
            head_data: [0u8; message::MSG_HEAD_SIZE],
        })
    }

    #[inline]
    pub fn get_msg_data_count(&self) -> u16 {
        self.vec_deque.len() as u16
    }

    pub fn get_msg_data_vec_deque(&mut self) -> VecDeque<Vec<u8>> {
        std::mem::replace(&mut self.vec_deque, VecDeque::new())
    }

    #[inline]
    pub fn add_msg_data(&mut self, msg_data: Vec<u8>) -> Result<(), String> {
        if msg_data.len() > 0 && msg_data.len() <= self.max_size {
            Ok(self.vec_deque.push_back(msg_data))
        } else {
            Err(String::from("msg_data byte size error"))
        }
    }

    pub fn write(&mut self, socket: &mut TcpStream) -> WriteResult {
        'go_while: while let Some(msg_data) = self.vec_deque.front() {
            //create head data
            if self.is_write_finish_current_data {
                self.is_write_finish_current_data = false;
                //------------------encode head data start-----------------------------
                let data_size = msg_data.len() as u32;
                let size_and_mid = (data_size << 12) + self.next_mid as u32;
                bytes::write_u32(&mut self.head_data[..], size_and_mid);
                //------------------encode head data end-----------------------------

                if self.next_mid == message::MSG_MAX_ID {
                    self.next_mid = 0
                } else {
                    self.next_mid += 1;
                }
            }

            // write head data
            if self.head_pos < message::MSG_HEAD_SIZE {
                loop {
                    match socket.write(&self.head_data[self.head_pos..]) {
                        Ok(0) => {
                            //已写满缓冲区 不能再写到缓存区
                            return WriteResult::BufferFull;
                        }
                        Ok(size) => {
                            self.head_pos += size;
                            if self.head_pos == message::MSG_HEAD_SIZE {
                                //已写完 head_data
                                if msg_data.len() > 0 {
                                    break;
                                } else {
                                    self.head_pos = 0;
                                    //当前buffer没有数据
                                    self.vec_deque.pop_front();
                                    self.is_write_finish_current_data = true;
                                    break 'go_while;
                                }
                            } else {
                                //已写满缓冲区 不能再写到缓存区
                                return WriteResult::BufferFull;
                            }
                        }
                        Err(err) if err.kind() == ErrorKind::Interrupted => {
                            continue; //系统中断 再写一次
                        }
                        Err(err) => return WriteResult::Error(err.to_string()),
                    }
                }
            }
            //write buffer data
            if self.data_pos < msg_data.len() {
                loop {
                    match socket.write(&msg_data[self.data_pos..]) {
                        Ok(0) => {
                            //已写满缓冲区 不能再写到缓存区
                            return WriteResult::BufferFull;
                        }
                        Ok(size) => {
                            self.data_pos += size;
                            if self.data_pos == msg_data.len() {
                                self.head_pos = 0;
                                self.data_pos = 0;
                                //已写完当前buffer所有数据
                                self.vec_deque.pop_front();
                                self.is_write_finish_current_data = true;
                                break 'go_while; //已写完
                            } else {
                                //已写满缓冲区 不能再写到缓存区
                                return WriteResult::BufferFull;
                            }
                        }
                        Err(err) if err.kind() == ErrorKind::Interrupted => {
                            continue; //系统中断 再写一次
                        }
                        Err(err) => return WriteResult::Error(err.to_string()),
                    }
                }
            }
        }
        WriteResult::Finish
    }
}
*/