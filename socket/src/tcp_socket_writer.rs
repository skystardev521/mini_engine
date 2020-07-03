use crate::message::MsgData;
use crate::tcp_socket::WriteResult;
use crate::tcp_socket_const;
use std::collections::VecDeque;
use std::io::prelude::Write;
use std::io::ErrorKind;
use std::net::TcpStream;
use utils::bytes;

pub struct TcpSocketWriter {
    next_id: u16,
    head_pos: usize,
    data_pos: usize,
    max_size: usize,
    vec_deque: VecDeque<Box<MsgData>>,
    is_write_finish_current_data: bool,
    head_data: [u8; tcp_socket_const::MSG_HEAD_SIZE],
}

impl TcpSocketWriter {
    pub fn new(max_size: u32) -> Box<Self> {
        Box::new(TcpSocketWriter {
            next_id: 0,
            head_pos: 0,
            data_pos: 0,
            vec_deque: VecDeque::new(),
            is_write_finish_current_data: true,
            head_data: [0u8; tcp_socket_const::MSG_HEAD_SIZE],
            max_size: if max_size > tcp_socket_const::MSG_MAX_SIZE {
                tcp_socket_const::MSG_MAX_SIZE
            } else {
                max_size
            } as usize,
        })
    }

    #[inline]
    pub fn get_msg_data_count(&self) -> u16 {
        self.vec_deque.len() as u16
    }

    #[inline]
    pub fn add_msg_data(&mut self, msg_data: Box<MsgData>) -> Result<(), String> {
        if msg_data.data.len() <= self.max_size {
            Ok(self.vec_deque.push_back(msg_data))
        } else {
            Err(String::from("MsgSizeTooBig"))
        }
    }

    pub fn write(&mut self, socket: &mut TcpStream) -> WriteResult {
        'go_while: while let Some(msg_data) = self.vec_deque.front() {
            //create head data
            if self.is_write_finish_current_data {
                self.is_write_finish_current_data = false;
                //------------------encode head data start-----------------------------
                let data_size = msg_data.data.len() as u32;
                let size_and_id = (data_size << 12) + self.next_id as u32;
                bytes::write_u32(&mut self.head_data[..], size_and_id);
                bytes::write_u16(
                    &mut self.head_data[tcp_socket_const::HEAD_DATA_ID_POS..],
                    msg_data.id,
                );
                //------------------encode head data end-----------------------------

                if self.next_id == tcp_socket_const::MSG_MAX_ID {
                    self.next_id = 0
                } else {
                    self.next_id += 1;
                }
            }

            // write head data
            if self.head_pos < tcp_socket_const::MSG_HEAD_SIZE {
                loop {
                    match socket.write(&self.head_data[self.head_pos..]) {
                        Ok(0) => {
                            //已写满缓冲区 不能再写到缓存区
                            return WriteResult::BufferFull;
                        }
                        Ok(size) => {
                            self.head_pos += size;
                            if self.head_pos == tcp_socket_const::MSG_HEAD_SIZE {
                                //已写完 head_data
                                if msg_data.data.len() > 0 {
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
                        Err(err) => return WriteResult::Error(format!("{}", err)),
                    }
                }
            }
            //write buffer data
            if self.data_pos < msg_data.data.len() {
                loop {
                    match socket.write(&msg_data.data[self.data_pos..]) {
                        Ok(0) => {
                            //已写满缓冲区 不能再写到缓存区
                            return WriteResult::BufferFull;
                        }
                        Ok(size) => {
                            self.data_pos += size;
                            if self.data_pos == msg_data.data.len() {
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
                        Err(err) => return WriteResult::Error(format!("{}", err)),
                    }
                }
            }
        }
        WriteResult::Finish
    }
}
