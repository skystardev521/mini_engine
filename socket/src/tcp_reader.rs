use crate::message;
use crate::message::MsgData;
use crate::message::MsgHead;
use log::info;
use std::io::prelude::Read;
use std::io::ErrorKind;
use std::mem;
use std::net::TcpStream;
use utils::bytes;

#[derive(Debug, PartialEq)]
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
    maxsize: u32,
    headpos: usize,
    msgdata: Box<MsgData>,
    headdata: [u8; message::MSG_HEAD_SIZE],
    msgdatacb: fn(Box<MsgData>),
}

impl TcpReader {
    /// maxsize：消息的最大字节1024 * 1024
    /// msgdatacb：有新消息回调这个函数
    pub fn new(maxsize: u32, msgdatacb: fn(Box<MsgData>)) -> Box<Self> {
        Box::new(TcpReader {
            id: 0,
            headpos: 0,
            headdata: [0u8; message::MSG_HEAD_SIZE],
            msgdatacb: msgdatacb,
            msgdata: Box::new(MsgData {
                id: 0,
                data: vec![0u8; 0],
            }),
            maxsize: if maxsize > message::MSG_MAX_SIZE {
                message::MSG_MAX_SIZE
            } else {
                maxsize
            },
        })
    }

    #[inline]
    fn deserialize_head(&self) -> MsgHead {
        let data = bytes::read_u32(&self.headdata);
        MsgHead {
            id: (data << 20 >> 20) as u16,
            datasize: (data >> 12) as u32,
            dataid: bytes::read_u16(&self.headdata[4..]),
        }
    }

    #[inline]
    fn check_head(&self, head: &MsgHead) -> EnumResult {
        if head.dataid != self.id {
            return EnumResult::MsgPackIdError;
        }
        if head.datasize > self.maxsize {
            return EnumResult::MsgSizeTooBig;
        }
        EnumResult::OK
    }

    #[inline]
    fn id_increment(&mut self) {
        self.id += 1;
        if self.id > message::MSG_MAX_ID {
            self.id = 0;
        }
    }

    pub fn read(&mut self, stream: &mut TcpStream) -> Result<EnumResult, String> {
        loop {
            if self.headpos != message::MSG_HEAD_SIZE {
                loop {
                    match stream.read(&mut self.headdata[self.headpos..]) {
                        Ok(0) => return Ok(EnumResult::ReadZeroSize),
                        Ok(size) => {
                            self.headpos += size;
                            //读取到的字节数
                            if self.headpos == message::MSG_HEAD_SIZE {
                                let head = self.deserialize_head();
                                let result = self.check_head(&head);
                                if result != EnumResult::OK {
                                    return Ok(result);
                                } else {
                                    self.id_increment();
                                }

                                if head.datasize == 0 {
                                    //读完一个包
                                    self.headpos = 0;
                                    (self.msgdatacb)(Box::new(MsgData {
                                        id: head.dataid,
                                        data: vec![],
                                    }));
                                    continue;
                                } else {
                                    self.msgdata = Box::new(MsgData {
                                        id: head.dataid,
                                        data: vec![0u8; head.datasize as usize],
                                    });
                                    break; //读完包头数据 go to read buffer data
                                }
                            }
                            //缓冲区已读完 包头数据 还没有读完
                            return Ok(EnumResult::BufferIsEmpty);
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                            info!("ErrorKind::WouldBlock");
                            //缓冲区已读完 包头数据 还没有读完
                            return Ok(EnumResult::BufferIsEmpty);
                        }
                        Err(ref err) if err.kind() == ErrorKind::Interrupted => {
                            info!("ErrorKind::Interrupted");
                            continue; ////系统中断 再read一次
                        }
                        Err(err) => return Err(format!("{}", err)),
                    }
                }
            }

            loop {
                //read buffer data
                let datapos = self.msgdata.data.len();
                match stream.read(&mut self.msgdata.data[datapos..]) {
                    Ok(0) => return Ok(EnumResult::ReadZeroSize),
                    Ok(_size) => {
                        //读取到的字节数
                        if self.msgdata.data.len() == self.msgdata.data.capacity() {
                            //读完一个包
                            self.headpos = 0;
                            let newmsgdata = Box::new(MsgData {
                                id: 0,
                                data: vec![],
                            });
                            (self.msgdatacb)(mem::replace(&mut self.msgdata, newmsgdata));
                            break;
                        } else {
                            //缓冲区已读完 buffer数据 还没有读完
                            return Ok(EnumResult::BufferIsEmpty);
                        }
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                        info!("ErrorKind::WouldBlock");
                        //缓冲区已读完 包头数据 还没有读完
                        return Ok(EnumResult::BufferIsEmpty);
                    }
                    Err(ref err) if err.kind() == ErrorKind::Interrupted => {
                        info!("ErrorKind::Interrupted");
                        continue; ////系统中断 再read一次
                    }
                    Err(err) => return Err(format!("{}", err)),
                }
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
        let headdata: u32 = u8s_to_val(&self.buffer[self.out_pos..]);
        let headdata = headdata as usize;
        self.body_size = headdata >> 12;
        let id = (headdata << 20 >> 20) as u16;
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
//}
