use crate::entity::NetData;

use std::io::prelude::Read;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;

use utils::bytes;

//包ID最大值
const MAX_ID: u16 = 4096;
///数据包头长度 6 个字节
///(包体字节数 13~32位)+(包Id 1~12位) + 任务Id
const HEAD_SIZE: usize = 6;

pub enum EnumResult {
    OK,
    ReadZeroSize,
    MsgSizeTooBig,
    BufferIsEmpty,
    MsgPackIdError,
}

pub struct TcpReader {
    //包id(0~4096)
    id: u16,
    head_pos: usize,
    body_max_size: usize,
    net_data: NetData,
    head_data: [u8; HEAD_SIZE],
}

impl TcpReader {
    ///err:-1, 256 <= msg_max_size <= 1024 * 1024
    pub fn new(msg_max_size: u32) -> Box<Self> {
        Box::new(TcpReader {
            id: 0,
            head_pos: 0,
            head_data: [0u8; HEAD_SIZE],
            body_max_size: msg_max_size as usize - HEAD_SIZE,
            net_data: /*Box::new(*/NetData {
                id: 0,
                buffer: vec![0u8; 0],
            }/*)*/,
        })
    }

    pub fn read(
        &mut self,
        stream: &mut TcpStream,
        net_data_cb: fn(Box<NetData>),
    ) -> Result<EnumResult, Error> {
        loop {
            if self.head_pos != HEAD_SIZE {
                loop {
                    let head_pos = self.head_data.len();
                    match stream.read(&mut self.head_data[head_pos..]) {
                        Ok(0) => {
                            return Ok(EnumResult::ReadZeroSize);
                        }
                        Ok(_size) => {
                            //读取到的字节数
                            if self.head_data.len() == HEAD_SIZE {
                                let data = bytes::read_u32(&self.head_data);
                                let pack_id = (data << 20 >> 20) as u16;

                                if pack_id != self.id {
                                    return Ok(EnumResult::MsgPackIdError);
                                }
                                self.id += 1;

                                let buffer_size = (data >> 12) as usize;
                                if buffer_size > self.body_max_size {
                                    return Ok(EnumResult::MsgSizeTooBig);
                                }

                                let id = bytes::read_u16(&self.head_data[4..]);
                                if buffer_size == 0 {
                                    //读完一个包
                                    self.head_pos = 0;
                                    net_data_cb(Box::new(NetData {
                                        id: id,
                                        buffer: vec![0u8; 0],
                                    }));
                                    continue;
                                } else {
                                    self.net_data = /*Box::new(*/NetData {
                                        id: id,
                                        buffer: vec![0u8; buffer_size],
                                    }/*)*/;
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
                let net_data = self.net_data;
                let buffer_pos = net_data.buffer.len();
                match stream.read(&mut net_data.buffer[buffer_pos..]) {
                    Ok(0) => {
                        println!("ErrorKind::Interrupted");
                        println!("stream.read result 0");
                        return Ok(EnumResult::ReadZeroSize);
                    }
                    Ok(_size) => {
                        //读取到的字节数
                        if net_data.buffer.len() == net_data.buffer.capacity() {
                            //读完一个包
                            net_data_cb(Box::new(net_data));
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

    /*
    ///包头上长度为4字节
    ///1~12位(包Id)
    ///13~32位(包体字节数)
    fn split_pack(&mut self) -> Result<(), ErrorKind> {
        if self.body_size == 0 {
            if self.in_pos - self.out_pos < HEAD_SIZE {
                return Ok(());
            }
            let head_data: u32 = u8s_to_val(&self.buffer[self.out_pos..]);
            let head_data = head_data as usize;
            self.body_size = head_data >> 12;
            let id = (head_data << 20 >> 20) as u16;
            if id != self.id {
                return Err(ErrorKind::InvalidData);
            }

            if self.body_size == 0 {

                //return Err(ErrorKind::InvalidData);
            }

            if MAX_ID == self.id {
                self.id = 0;
            } else {
                self.id += 1;
            }

            if self.body_size > self.buffer_size - HEAD_SIZE {
                return Err(ErrorKind::InvalidData);
            }
        }

        let pack_size = self.body_size + HEAD_SIZE;
        let data_size = self.in_pos - self.out_pos;
        if data_size < pack_size {
            if self.out_pos + pack_size > self.buffer_size {
                //把数据移动到 self.out_pos = 0;
                unsafe {
                    ptr::copy(
                        self.buffer.as_ptr().add(self.out_pos),
                        self.buffer.as_mut_ptr().add(0),
                        data_size,
                    );
                }
                self.out_pos = 0;
                self.in_pos = data_size;
            }
            return Ok(());
        }

        if data_size == pack_size {
            //一个完整的包
            //self.new_task(&self.buffer[self.out_pos..self.in_pos]);
            self.out_pos = 0;
            self.in_pos = 0;

            return Ok(());
        }
        //data_size 大于一个完整的包
        let middle_pos = self.out_pos + pack_size;
        //self.new_task(&self.buffer[self.out_pos..middle_pos]);

        self.out_pos = middle_pos;
        self.split_pack()
    }

    fn new_task(&mut self, buffer: &[u8]) {
        if 2 == buffer.len() {
            let mut net_task = NetTask {
                id: 0,
                buffer: vec![0u8; 1],
            };
        }

        self.body_size = 0; //
                            //(self.pack_cb)(&self.buffer[self.out_pos..self.in_pos]);
                            /*
                            unsafe {
                                std::ptr::copy_nonoverlapping(n.as_ptr(), buffer[0..len].as_mut_ptr(), len);
                            }
                            */
    }

    pub fn read(&mut self, stream: &mut TcpStream) -> Result<(), ErrorKind> {
        loop {
            match stream.read(&mut self.buffer[self.in_pos..]) {
                Ok(0) => {
                    println!("ErrorKind::Interrupted");
                    println!("stream.read result 0");
                    //return Err(ErrorKind::ConnectionAborted);
                }
                Ok(size) => {
                    //读取到的字节数

                    self.in_pos += size;
                    match self.split_pack() {
                        Ok(()) => continue,
                        Err(e) => return Err(e),
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                    println!("ErrorKind::Interrupted");
                    continue; ////系统中断 再read一次
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    println!("ErrorKind::WouldBlock");
                    continue; ////系统中断 再read一次
                }
                Err(e) => return Err(e.kind()),
            }
        }
    }
    */
}
