use std::collections::LinkedList;
use std::io::prelude::Write;
use std::io::ErrorKind;
use std::net::TcpStream;

use crate::bytes;
use crate::nettask::NetTask;

const MAX_ID: u16 = 4096;
///数据包头长度 6 个字节
///(包体字节数 13~32位)+(包Id 1~12位)+任务Id
const HEAD_SIZE: usize = 6;

pub struct TcpWriter {
    id: u16,
    //有任务未完成
    have_task: bool,
    head_pos: usize,
    head_data: [u8; HEAD_SIZE],
    //已发送的字节
    buffer_pos: usize,
    list: LinkedList<Box<NetTask>>,
}

impl TcpWriter {
    pub fn new() -> Box<Self> {
        Box::new(TcpWriter {
            id: 0,
            head_pos: 0,
            buffer_pos: 0,
            have_task: false,
            list: LinkedList::new(),
            head_data: [0u8; HEAD_SIZE],
        })
    }

    pub fn task_num(&self) -> usize {
        self.list.len()
    }

    pub fn add_task(&mut self, task: Box<NetTask>) {
        self.list.push_back(task);
    }

    pub fn loop_write(&mut self, stream: &mut TcpStream) -> Result<(), ErrorKind> {
        while let Some(task) = self.list.front() {
            if self.have_task == false {
                let id = self.id;
                self.head_pos = 0;
                self.buffer_pos = 0;
                self.have_task = true;
                self.id = if id == MAX_ID { 0 } else { self.id + 1 };

                let data_size = task.buffer.len() as u32;
                let data_size_id = data_size << 12 + id as u32;

                bytes::write_u32(&mut self.head_data[0..], data_size_id);
                bytes::write_u16(&mut self.head_data[4..], task.id);
            }

            if self.head_pos < HEAD_SIZE {
                loop {
                    match stream.write(&self.head_data[self.head_pos..]) {
                        Ok(0)=>
                            //已写满缓冲区 不能再写到缓存区
                            return Err(ErrorKind::WriteZero);
                        Ok(size) => {
                            self.head_pos += size;
                            if self.head_pos == HEAD_SIZE {
                                break; //已写完 head_data
                            }
                            //已写满缓冲区 不能再写到缓存区
                            return Err(ErrorKind::WriteZero);
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                            continue; //系统中断 再写一次
                        }
                        Err(ref e) => return Err(e.kind()),
                    }
                }
            }
            loop {
                match stream.write(&task.buffer[self.buffer_pos..]) {
                    Ok(size) => {
                        self.buffer_pos += size;
                        if self.buffer_pos == task.buffer.len() {
                            //已写完当前buffer所有数据
                            self.have_task = true;
                            self.list.pop_front();
                            break;
                        }
                        //已写满缓冲区 不能再写到缓存区
                        return Err(ErrorKind::WriteZero);
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                        continue; //系统中断 再写一次
                    }
                    Err(ref e) => return Err(e.kind()),
                }
            }
        }
        Ok(())
    }
}
