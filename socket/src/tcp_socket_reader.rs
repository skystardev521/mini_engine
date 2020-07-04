use crate::message::MsgData;
use crate::tcp_socket::ReadResult;
use crate::tcp_socket_const;
use std::io::prelude::Read;
use std::io::ErrorKind;
use std::mem;
use std::net::TcpStream;
use utils::bytes;

pub struct TcpSocketReader {
    //包id(0~4096)
    next_id: u16,
    max_size: u32,
    head_pos: usize,
    data_pos: usize,
    msg_data: Box<MsgData>,
    head_data: [u8; tcp_socket_const::MSG_HEAD_SIZE],
}

impl TcpSocketReader {
    /// max_size：消息的最大字节1024 * 1024
    pub fn new(max_size: u32) -> Box<Self> {
        Box::new(TcpSocketReader {
            next_id: 0,
            head_pos: 0,
            data_pos: 0,
            head_data: [0u8; tcp_socket_const::MSG_HEAD_SIZE],
            msg_data: Box::new(MsgData {
                id: 0,
                data: vec![],
            }),
            max_size: if max_size > tcp_socket_const::MSG_MAX_SIZE {
                tcp_socket_const::MSG_MAX_SIZE
            } else {
                max_size
            },
        })
    }

    pub fn read(&mut self, socket: &mut TcpStream) -> ReadResult {
        if self.head_pos != tcp_socket_const::MSG_HEAD_SIZE {
            loop {
                match socket.read(&mut self.head_data[self.head_pos..]) {
                    Ok(0) => return ReadResult::ReadZeroSize,
                    Ok(size) => {
                        self.head_pos += size;
                        //读取到的字节数
                        if self.head_pos == tcp_socket_const::MSG_HEAD_SIZE {
                            //--------------------decode msg head start-----------------
                            let new_data = bytes::read_u32(&self.head_data);
                            let new_data_id = bytes::read_u16(
                                &self.head_data[tcp_socket_const::HEAD_DATA_ID_POS..],
                            );

                            let new_id = (new_data << 20 >> 20) as u16;
                            let new_data_size = (new_data >> 12) as u32;
                            //--------------------decode msg head end-------------------

                            if new_id != self.next_id {
                                return ReadResult::Error("Msg Id Error".into());
                            }
                            if new_data_size > self.max_size {
                                return ReadResult::Error(format!(
                                    "socket.read Msg Size Too Big size:{}",
                                    new_data_size
                                ));
                            }

                            if self.next_id == tcp_socket_const::MSG_MAX_ID {
                                self.next_id = 0;
                            } else {
                                self.next_id += 1;
                            }

                            if new_data_size == 0 {
                                //读完一个包
                                self.head_pos = 0;
                                return ReadResult::Data(Box::new(MsgData {
                                    id: new_data_id,
                                    data: vec![],
                                }));
                            } else {
                                //读完包头数据
                                self.msg_data = Box::new(MsgData {
                                    id: new_data_id,
                                    data: vec![0u8; new_data_size as usize],
                                });
                                break;
                            }
                        }
                        //缓冲区已读完 包头数据 还没有读完
                        return ReadResult::BufferIsEmpty;
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                        //缓冲区已读完 包头数据 还没有读完
                        return ReadResult::BufferIsEmpty;
                    }
                    Err(ref err) if err.kind() == ErrorKind::Interrupted => {
                        continue; ////系统中断 再read一次
                    }
                    Err(err) => return ReadResult::Error(format!("{}", err)),
                }
            }
        }

        loop {
            //read msg data
            match socket.read(&mut self.msg_data.data[self.data_pos..]) {
                Ok(0) => return ReadResult::ReadZeroSize,
                Ok(size) => {
                    //读取到的字节数
                    self.data_pos += size;
                    if self.data_pos < self.msg_data.data.capacity() {
                        //tcp缓冲区已读完 数据还没有读完
                        return ReadResult::BufferIsEmpty;
                    }
                    //读完一个包
                    self.head_pos = 0;
                    self.data_pos = 0;
                    let newmsg_data = Box::new(MsgData {
                        id: 0,
                        data: vec![],
                    });
                    return ReadResult::Data(mem::replace(&mut self.msg_data, newmsg_data));
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                    //info!("ErrorKind::WouldBlock");
                    //缓冲区已读完 包头数据 还没有读完
                    return ReadResult::BufferIsEmpty;
                }
                Err(ref err) if err.kind() == ErrorKind::Interrupted => {
                    //info!("ErrorKind::Interrupted");
                    continue; ////系统中断 再read一次
                }
                Err(err) => return ReadResult::Error(format!("{}", err)),
            }
        }
    }
}
