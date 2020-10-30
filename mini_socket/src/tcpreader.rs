/*

//use std::ffi::CStr;
use crate::nettask::NetTask;
use std::io::prelude::Read;
use std::io::ErrorKind;
use std::net::TcpStream;

use crate::bytes;
use crate::neterrkind::NetErrKind;

//包ID最大值
const MAX_ID: u16 = 4096;
///数据包头长度 6 个字节
///(包体字节数 13~32位)+(包Id 1~12位) + 任务Id
const HEAD_SIZE: usize = 6;
///数据最小字节数
const DATA_MIN_SIZE: usize = 256;
///数据最大字节数
const DATA_MAX_SIZE: usize = 1024 * 1024;
///数据包体最大字节数
const BODY_MAX_SIZE: usize = DATA_MAX_SIZE - HEAD_SIZE;

const MAX_ID: u16 = 4096;

///数据包头长度 6 个字节
///(包体字节数 13~32位)+(包Id 1~12位) + 任务Id
const HEAD_SIZE: usize = 6;

const DATA_MIN_SIZE: usize = 256;

const DATA_MAX_SIZE: usize = 1024 * 1024;
///buffer最少字节数
const BODY_MAX_SIZE: usize = DATA_MAX_SIZE - HEAD_SIZE;

pub struct TcpReader {
    //包id(0~4096)
    id: u16,
    head_pos: usize,
    head_data: [u8; HEAD_SIZE],

    buffer_pos: usize,

    buffer_max_size: usize,
    nettask: Option<Box<NetTask>>,
    //完整数据包时调用
    net_task_cb: fn(Box<NetTask>),
}

#[inline]
fn u8s_to_val<T>(u8s: &[u8]) -> T
where
    T: Copy,
{
    let p: *const u8 = u8s.as_ptr();
    #[cfg(target_endian = "little")]
    {
        unsafe { *(p as *const T) }
    }
    #[cfg(not(target_endian = "little"))]
    {
        unsafe { *(p as *const T) }.swap_bytes()
    }
}

fn new_task(buffer: &[u8]) -> (u16, NetTask) {
    let data = bytes::read_u32(buffer);
    let id = (data << 20 >> 20) as u16;
    let buffer_size = (data >> 12) as usize;
    (
        id,
        NetTask {
            id: bytes::read_u16(buffer),
            buffer: vec![0u8; buffer_size],
        },
    )
}

impl TcpReader {
    ///err:-1, 256 <= data_max_size <= 1024 * 1024
    pub fn new(data_max_size: usize, net_task_cb: fn(Box<NetTask>)) -> Result<Box<Self>, i8> {
        if data_max_size < DATA_MIN_SIZE {
            return Err(-1);
        }
        if data_max_size > DATA_MAX_SIZE {
            return Err(-1);
        }
        Ok(Box::new(TcpReader {
            id: 0,
            head_pos: 0,
            buffer_pos: 0,
            nettask: None,
            net_task_cb: net_task_cb,
            head_data: [0u8; HEAD_SIZE],
            buffer_max_size: data_max_size as usize,
        }))
    }
    fn new_task(&mut self) -> Result<(), ErrorKind> {
        let data = bytes::read_u32(&self.head_data);
        let id = (data << 20 >> 20) as u16;
        if id != self.id {
            return ();
        }
        let buffer_size = (data >> 12) as usize;

        NetTask {
            id: bytes::read_u16(&self.head_data[4..]),
            buffer: vec![0u8; buffer_size],
        }
    }

    pub fn read(&mut self, stream: &mut TcpStream) -> Result<(), ErrorKind> {
        loop {
            loop {
                match stream.read(&mut self.head_data[self.head_pos..]) {
                    Ok(0) => {
                        println!("ErrorKind::Interrupted");
                        println!("stream.read result 0");
                        return Err(ErrorKind::ConnectionAborted);
                    }
                    Ok(size) => {
                        //读取到的字节数
                        self.head_pos += size;
                        if self.head_pos == HEAD_SIZE {
                            let (id, task) = new_task(&self.head_data);
                            break; //读完包头数据
                        }
                        //缓冲区已读完 包头数据 还没有读完
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
            loop {
                match stream.read(&self.buffer[self.in_pos..]) {
                    Ok(0) => {
                        println!("ErrorKind::Interrupted");
                        println!("stream.read result 0");
                        return Err(ErrorKind::ConnectionAborted);
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
*/