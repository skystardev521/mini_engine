use mini_socket::tcp_buf_rw::ReadResult;
use mini_socket::tcp_buf_rw::TcpBufRw;
use mini_socket::tcp_buf_rw::WriteResult;
use mini_utils::bytes;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::mem;
use std::net::TcpStream;
use std::ptr;

use crate::net_message::LanNetMsg;

/// Msg Id最大值
pub const MSG_MAX_ID: u16 = 4095;

///数据包头长度4个字节
/// msg id: 0 ~ 4095
/// data size: 0 ~ (1024 * 1024)
/// |data size:13~32位|+|MID:1~12位|
pub const MSG_HEAD_SIZE: usize = 4;

/// 数据包体最大字节数
pub const MSG_MAX_SIZE: usize = 1024 * 1024;

pub struct LanBufRw {
    buf_reader: bufReader,
    buf_writer: bufWriter,
}

pub struct bufReader {
    //包id(0~4096)
    id: u16,
    /// 0:no data
    head_pos: usize,
    /// 0:no data
    body_pos: usize,
    body_data: Vec<u8>,
    head_data: [u8; MSG_HEAD_SIZE],
}

pub struct bufWriter {
    //包id(0~4096)
    id: u16,
    body_pos: usize,
    head_pos: usize,
    is_fill_head: bool,
    head_data: [u8; MSG_HEAD_SIZE],
}

impl Default for LanBufRw {
    fn default() -> Self {
        LanBufRw {
            buf_reader: bufReader {
                id: 0,
                body_pos: 0,
                head_pos: 0,
                body_data: vec![],
                head_data: [0u8; MSG_HEAD_SIZE],
            },
            buf_writer: bufWriter {
                id: 0,
                body_pos: 0,
                head_pos: 0,
                is_fill_head: false,
                head_data: [0u8; MSG_HEAD_SIZE],
            },
        }
    }
}

#[inline]
fn next_id(id: u16) -> u16 {
    if id == MSG_MAX_ID {
        0
    } else {
        id + 1
    }
}

#[inline]
fn split_head(buffer: &[u8]) -> (u16, usize) {
    let u32_val = bytes::read_u32(buffer);
    //消息id                      //消息字节
    ((u32_val << 20 >> 20) as u16, (u32_val >> 12) as usize)
}

#[inline]
fn fill_head(id: u16, msize: usize, buffer: &mut [u8]) {
    let u32_val = (msize as u32) << 12;
    bytes::write_u32(buffer, u32_val + id as u32);
}

fn write_data(buffer: &[u8], wsize: &mut usize, socket: &mut TcpStream) -> WriteResult {
    loop {
        match socket.write(&buffer) {
            Ok(0) => {
                return WriteResult::Error("disconnect".into());
            }
            Ok(size) => {
                if size == buffer.len() {
                    return WriteResult::Finish;
                } else {
                    *wsize += size;
                    return WriteResult::BufferFull;
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                return WriteResult::BufferFull;
            }
            Err(ref err) if err.kind() == ErrorKind::Interrupted => {
                continue; //系统中断 write
            }
            Err(ref err) => return WriteResult::Error(err.to_string()),
        }
    }
}

impl TcpBufRw<LanNetMsg> for LanBufRw {
    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, msg: &mut LanNetMsg) -> WriteResult {
        if MSG_MAX_SIZE < msg.buff.len() {
            return WriteResult::Error("msg size error".into());
        }
        let bw = &mut self.buf_writer;

        // 新的消息包
        if bw.head_pos == 0 && !bw.is_fill_head {
            bw.is_fill_head = true;
            fill_head(bw.id, msg.buff.len(), &mut bw.head_data);
            bw.id = next_id(bw.id);
        }

        // 写头部数据
        if bw.head_pos < MSG_HEAD_SIZE {
            // 写成功的字节数
            let mut wsize = 0;
            let result = write_data(&bw.head_data[bw.head_pos..], &mut wsize, socket);
            if result == WriteResult::Finish {
                bw.head_pos = MSG_HEAD_SIZE;
            } else {
                if WriteResult::BufferFull == result {
                    bw.head_pos += wsize;
                }
                return result;
            }
        }
        // 写成功的字节数
        let mut wsize = 0;
        // 把包休数据写入
        let result = write_data(&msg.buff[bw.body_pos..], &mut wsize, socket);
        if WriteResult::Finish == result {
            bw.head_pos = 0;
            bw.body_pos = 0;
            bw.is_fill_head = false;
        }
        if WriteResult::BufferFull == result {
            bw.body_pos += wsize;
        }
        return result;
    }

    /// 从tcp bufferfer中读取数据
    /// buffer: 共享缓冲区 这方式用于读小包的方案
    fn read(&mut self, socket: &mut TcpStream, buffer: &mut Vec<u8>) -> ReadResult<LanNetMsg> {
        let mut in_pos = 0;
        let mut vec_msg: Vec<Vec<u8>> = vec![];
        let br = &mut self.buf_reader;

        loop {
            match socket.read(&mut buffer[in_pos..]) {
                Ok(0) => {
                    return ReadResult::Error(vec_msg, "disconnect".into());
                }
                Ok(size) => {
                    in_pos += size;
                    
                    // 读完了TCP缓存区数据
                    if in_pos < buffer.capacity() {
                        match br.split_msg_data(in_pos, buffer, &mut vec_msg) {
                            None => {
                                return ReadResult::Data(vec_msg);
                            }
                            Some(err) => {
                                return ReadResult::Error(vec_msg, err);
                            }
                        }
                    }
                    if let Some(err) = br.split_msg_data(in_pos, buffer, &mut vec_msg) {
                        return ReadResult::Error(vec_msg, err);
                    }
                    in_pos = 0; // 重新开始读到buffer中
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                    match br.split_msg_data(in_pos, buffer, &mut vec_msg) {
                        None => {
                            return ReadResult::Data(vec_msg);
                        }
                        Some(err) => {
                            return ReadResult::Error(vec_msg, err);
                        }
                    }
                }
                Err(ref err) if err.kind() == ErrorKind::Interrupted => {
                    continue; ////系统中断 再read一次
                }
                Err(ref err) => return ReadResult::Error(vec_msg, err.to_string()),
            }
        }
    }
}

impl bufReader {
    fn split_msg_data(
        &mut self,
        in_pos: usize,
        buffer: &Vec<u8>,
        vec_msg: &mut Vec<Vec<u8>>,
    ) -> Option<String> {
        let mut out_pos = 0;
        loop {
            if self.head_pos == 0 {
                let buffer_data_len = in_pos - out_pos;
                //不够包头数据把数据存到head_data
                if buffer_data_len < MSG_HEAD_SIZE {
                    self.head_pos = buffer_data_len;
                    unsafe {
                        ptr::copy_nonoverlapping(
                            buffer[out_pos..].as_ptr(),
                            self.head_data[0..].as_mut_ptr(),
                            buffer_data_len,
                        );
                    }
                    return None;
                }
                self.head_pos = MSG_HEAD_SIZE;
                let head_data = &buffer[out_pos..];
                out_pos += MSG_HEAD_SIZE;

                //获取包头数据
                let (id, body_size) = split_head(&head_data);
                if let Some(err) = self.check_head_data(id, body_size) {
                    return Some(err);
                }

                //分配包体内存
                self.body_data = vec![0u8; body_size];

                let buffer_data_len = in_pos - out_pos;
                //没有包体数据
                if buffer_data_len == 0 {
                    self.body_pos = 0;
                    return None;
                }
                //不够包体数据
                if buffer_data_len < body_size {
                    self.body_pos = buffer_data_len;
                    //不够包体数据把数据存到body_data
                    unsafe {
                        ptr::copy_nonoverlapping(
                            buffer[out_pos..].as_ptr(),
                            self.body_data[0..].as_mut_ptr(),
                            buffer_data_len,
                        );
                    }
                    return None;
                }
                //足够包体数据
                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer[out_pos..].as_ptr(),
                        self.body_data[0..].as_mut_ptr(),
                        body_size,
                    );
                }
                self.head_pos = 0;
                out_pos += body_size;
                vec_msg.push(mem::replace(&mut self.body_data, vec![]));

                continue;
            }

            if self.head_pos < MSG_HEAD_SIZE {
                //----------------------------------------------------------------------
                let buffer_data_len = in_pos - out_pos;
                let head_tail_len = MSG_HEAD_SIZE - self.head_pos;
                //不够包头长度把数据存到head_data
                if buffer_data_len < head_tail_len {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            buffer[out_pos..].as_ptr(),
                            self.head_data[self.head_pos..].as_mut_ptr(),
                            buffer_data_len,
                        );
                    }
                    self.head_pos += buffer_data_len;
                    return None;
                }

                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer[out_pos..].as_ptr(),
                        self.head_data[self.head_pos..].as_mut_ptr(),
                        head_tail_len,
                    );
                }
                out_pos += head_tail_len;
                self.head_pos = MSG_HEAD_SIZE;
                //----------------------------------------------------------------------
                //获取包头数据
                let (id, msg_size) = split_head(&self.head_data);
                if let Some(err) = self.check_head_data(id, msg_size) {
                    return Some(err);
                }

                //分配包体内存
                self.body_data = vec![0u8; msg_size];

                let buffer_data_len = in_pos - out_pos;
                //没有包体数据
                if buffer_data_len == 0 {
                    self.body_pos = 0;
                    return None;
                }
                //不够包体数据
                if buffer_data_len < msg_size {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            buffer[out_pos..].as_ptr(),
                            self.body_data[0..].as_mut_ptr(),
                            buffer_data_len,
                        );
                    }
                    self.body_pos = buffer_data_len;
                    return None;
                }
                //足够包体数据
                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer[out_pos..].as_ptr(),
                        self.body_data[0..].as_mut_ptr(),
                        msg_size,
                    );
                }
                self.head_pos = 0;
                out_pos += msg_size;
                vec_msg.push(mem::replace(&mut self.body_data, vec![]));
                continue;
            }
            //--------------------把数据存到head_data
            let buffer_data_len = in_pos - out_pos;
            let body_data_tail = self.body_data.len() - self.body_pos;
            //不够包体数据
            if buffer_data_len < body_data_tail {
                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer[out_pos..].as_ptr(),
                        self.body_data[self.body_pos..].as_mut_ptr(),
                        buffer_data_len,
                    );
                }
                self.body_pos += buffer_data_len;
                return None;
            }
            //足够包体数据
            unsafe {
                ptr::copy_nonoverlapping(
                    buffer[out_pos..].as_ptr(),
                    self.body_data[self.body_pos..].as_mut_ptr(),
                    body_data_tail,
                );
            }
            self.head_pos = 0;
            out_pos += body_data_tail;
            vec_msg.push(mem::replace(&mut self.body_data, vec![]));
        }
    }

    #[inline]
    fn check_head_data(&mut self, id: u16, body_size: usize) -> Option<String> {
        if id != self.id {
            return Some("Msg Id Error".into());
        }

        self.id = next_id(self.id);

        if body_size == 0 {
            return Some("Msg Size is 0".into());
        }
        if body_size > MSG_MAX_SIZE {
            return Some(format!("Msg Size:{} Too Large", body_size));
        }
        return None;
    }
}

/*
impl bufReader {
    fn split_msg_data(
        &mut self,
        in_pos: usize,
        buffer: &Vec<u8>,
        vec_msg: &mut Vec<Vec<u8>>,
    ) -> Option<String> {
        let mut out_pos = 0;
        loop {
            if self.head_pos == 0 {
                let buffer_data_len = in_pos - out_pos;
                //不够包头数据把数据存到head_data
                if buffer_data_len < MSG_HEAD_SIZE {
                    self.head_pos = buffer_data_len;
                    unsafe {
                        ptr::copy_nonoverlapping(
                            buffer[out_pos..].as_ptr(),
                            self.head_data[0..].as_mut_ptr(),
                            buffer_data_len,
                        );
                    }
                    return None;
                }
                self.head_pos = MSG_HEAD_SIZE;
                let head_data = &buffer[out_pos..];
                out_pos += MSG_HEAD_SIZE;

                //获取包头数据
                let (id, msg_size) = split_head(&head_data);
                if let Some(err) = self.check_head_data(id, msg_size) {
                    return Some(err);
                }

                //分配包体内存
                self.body_data = vec![0u8; msg_size];

                let buffer_data_len = in_pos - out_pos;
                //没有包体数据
                if buffer_data_len == 0 {
                    self.body_pos = 0;
                    return None;
                }
                //不够包体数据
                if buffer_data_len < msg_size {
                    self.body_pos = buffer_data_len;
                    //不够包体数据把数据存到body_data
                    unsafe {
                        ptr::copy_nonoverlapping(
                            buffer[out_pos..].as_ptr(),
                            self.body_data[0..].as_mut_ptr(),
                            buffer_data_len,
                        );
                    }
                    return None;
                //足够包体数据
                }

                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer[out_pos..].as_ptr(),
                        self.body_data[0..].as_mut_ptr(),
                        msg_size,
                    );
                }
                self.head_pos = 0;
                out_pos += msg_size;
                vec_msg.push(mem::replace(&mut self.body_data, vec![]));

                continue;
            }

            if self.head_pos < MSG_HEAD_SIZE {
                //----------------------------------------------------------------------
                let buffer_data_len = in_pos - out_pos;
                let head_tail_len = MSG_HEAD_SIZE - self.head_pos;
                //不够包头长度把数据存到head_data
                if buffer_data_len < head_tail_len {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            buffer[out_pos..].as_ptr(),
                            self.head_data[self.head_pos..].as_mut_ptr(),
                            buffer_data_len,
                        );
                    }
                    self.head_pos += buffer_data_len;
                    return None;
                }

                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer[out_pos..].as_ptr(),
                        self.head_data[self.head_pos..].as_mut_ptr(),
                        head_tail_len,
                    );
                }
                out_pos += head_tail_len;
                self.head_pos = MSG_HEAD_SIZE;
                //----------------------------------------------------------------------
                //获取包头数据
                let (id, msg_size) = split_head(&self.head_data);
                if let Some(err) = self.check_head_data(id, msg_size) {
                    return Some(err);
                }

                //分配包体内存
                self.body_data = vec![0u8; msg_size];

                let buffer_data_len = in_pos - out_pos;
                //没有包体数据
                if buffer_data_len == 0 {
                    self.body_pos = 0;
                    return None;
                }
                //不够包体数据
                if buffer_data_len < msg_size {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            buffer[out_pos..].as_ptr(),
                            self.body_data[0..].as_mut_ptr(),
                            buffer_data_len,
                        );
                    }
                    self.body_pos = buffer_data_len;
                    return None;

                }
                //足够包体数据
                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer[out_pos..].as_ptr(),
                        self.body_data[0..].as_mut_ptr(),
                        msg_size,
                    );
                }
                self.head_pos = 0;
                out_pos += msg_size;
                vec_msg.push(mem::replace(&mut self.body_data, vec![]));
                               continue;
            }
            //--------------------把数据存到head_data
            let buffer_data_len = in_pos - out_pos;
            let body_data_tail = self.body_data.len() - self.body_pos;
            //不够包体数据
            if buffer_data_len < body_data_tail {
                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer[out_pos..].as_ptr(),
                        self.body_data[self.body_pos..].as_mut_ptr(),
                        buffer_data_len,
                    );
                }
                self.body_pos += buffer_data_len;
                return None;

            }
            //足够包体数据
            unsafe {
                ptr::copy_nonoverlapping(
                    buffer[out_pos..].as_ptr(),
                    self.body_data[self.body_pos..].as_mut_ptr(),
                    body_data_tail,
                );
            }
            self.head_pos = 0;
            out_pos += body_data_tail;
            vec_msg.push(mem::replace(&mut self.body_data, vec![]));
        }
    }

    #[inline]
    fn check_head_data(&mut self, id: u16, msg_size: usize) -> Option<String> {
        if id != self.id {
            println!("id:{} cur_id:{}", id, self.id);
            return Some("Msg Id Error".into());
        }

        self.id = next_id(self.id);

        if msg_size == 0 {
            return Some("Msg Size is 0".into());
        }
        if msg_size > MSG_MAX_SIZE {
            return Some(format!("Msg Size:{} Too Large", msg_size));
        }
        return None;
    }
}
*/
