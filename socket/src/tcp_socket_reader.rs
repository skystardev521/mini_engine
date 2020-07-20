use crate::message;
use crate::message::MsgData;
use std::io::prelude::Read;
use std::io::ErrorKind;
use std::mem;
use std::net::TcpStream;
use utils::bytes;

pub enum ReadResult {
    Error(String),
    ReadZeroSize,
    BufferIsEmpty,
    Data(Box<MsgData>),
}

pub struct TcpSocketReader {
    //包id(0~4096)
    next_mid: u16,
    max_size: usize,
    head_pos: usize,
    data_pos: usize,
    msg_data: Box<MsgData>,
    head_data: [u8; message::MSG_HEAD_SIZE],
}

impl TcpSocketReader {
    /// max_size：消息的最大字节1024 * 1024
    pub fn new(msg_max_size: u32) -> Box<Self> {
        let mut max_size = msg_max_size;
        if max_size > message::MSG_MAX_SIZE {
            max_size = message::MSG_MAX_SIZE
        }

        Box::new(TcpSocketReader {
            next_mid: 0,
            head_pos: 0,
            data_pos: 0,
            max_size: max_size as usize,
            head_data: [0u8; message::MSG_HEAD_SIZE],
            msg_data: Box::new(MsgData {
                pid: 0,
                ext: 0,
                data: vec![],
            }),
        })
    }

    pub fn read(&mut self, socket: &mut TcpStream) -> ReadResult {
        if self.head_pos != message::MSG_HEAD_SIZE {
            loop {
                match socket.read(&mut self.head_data[self.head_pos..]) {
                    Ok(0) => return ReadResult::ReadZeroSize,
                    Ok(size) => {
                        self.head_pos += size;
                        //读取到的字节数
                        if self.head_pos == message::MSG_HEAD_SIZE {
                            //--------------------decode msg head start-----------------
                            let new_data = bytes::read_u32(&self.head_data);
                            let pid =
                                bytes::read_u16(&self.head_data[message::HEAD_DATA_PID_POS..]);
                            let ext =
                                bytes::read_u32(&self.head_data[message::HEAD_DATA_EXT_POS..]);

                            let new_mid = (new_data << 20 >> 20) as u16;
                            let new_data_size = (new_data >> 12) as usize;

                            //--------------------decode msg head end-------------------

                            if new_mid != self.next_mid {
                                return ReadResult::Error("Msg Id Error".into());
                            }
                            if new_data_size > self.max_size {
                                return ReadResult::Error(format!(
                                    "Msg Max Size:{} read size:{}",
                                    self.max_size, new_data_size
                                ));
                            }

                            if self.next_mid == message::MSG_MAX_ID {
                                self.next_mid = 0;
                            } else {
                                self.next_mid += 1;
                            }

                            if new_data_size == 0 {
                                //读完一个包
                                self.head_pos = 0;
                                return ReadResult::Data(Box::new(MsgData {
                                    pid: pid,
                                    ext: ext,
                                    data: vec![],
                                }));
                            } else {
                                //读完包头数据
                                self.msg_data = Box::new(MsgData {
                                    pid: pid,
                                    ext: ext,
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
                    Err(err) => return ReadResult::Error(err.to_string()),
                }
            }
        }

        loop {
            //read msg data
            //如果一个连接在内网死循环发送消息 这里会卡住
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
                        pid: 0,
                        ext: 0,
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
                Err(err) => return ReadResult::Error(err.to_string()),
            }
        }
    }
}
