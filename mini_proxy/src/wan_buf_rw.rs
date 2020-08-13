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

/// Msg Id最大值
pub const MSG_MAX_ID: u16 = 4096;

///数据包头长度4个字节
/// msg id: 0 ~ 4095
/// data size: 0 ~ (1024 * 1024)
/// |data size:13~32位|+|MID:1~12位|
pub const MSG_HEAD_SIZE: usize = 4;

pub struct WanBufRw {
    max_size: usize,
    buf_reader: BufReader,
    buf_writer: BufWriter,
}

pub struct BufReader {
    //包id(0~4096)
    next_id: u16,
    /// 0:no data
    vec_pos: usize,
    /// 0:no data
    head_pos: usize,

    vec_data: Vec<u8>,

    head_data: [u8; MSG_HEAD_SIZE],
}

pub struct BufWriter {
    //包id(0~4096)
    next_id: u16,
    vec_pos: usize,
    head_pos: usize,
    head_data: [u8; MSG_HEAD_SIZE],
}

impl Default for WanBufRw {
    fn default() -> Self {
        WanBufRw {
            max_size: 1024,
            buf_reader: BufReader {
                next_id: 0,
                vec_pos: 0,
                head_pos: 0,
                vec_data: vec![],
                head_data: [0u8; MSG_HEAD_SIZE],
            },
            buf_writer: BufWriter {
                next_id: 0,
                vec_pos: 0,
                head_pos: 0,
                head_data: [0u8; MSG_HEAD_SIZE],
            },
        }
    }
}

#[inline]
fn split_head(buf: &[u8]) -> (u16, usize) {
    let u32_val = bytes::read_u32(buf);
    //消息id                      //消息字节
    ((u32_val << 20 >> 20) as u16, (u32_val >> 12) as usize)
}

#[inline]
fn fill_head(id: u16, msg_size: usize, buf: &mut [u8]) {
    let u32_val = (msg_size as u32) << 12;
    bytes::write_u32(buf, u32_val + id as u32);
}

fn write_data(buffer: &[u8], wsize: &mut usize, socket: &mut TcpStream) -> WriteResult {
    loop {
        match socket.write(&buffer) {
            Ok(0) => return WriteResult::Error("disconnect".into()),
            Ok(size) => {
                if size == buffer.len() {
                    return WriteResult::Finish;
                } else {
                    *wsize = size;
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

impl TcpBufRw<Vec<u8>> for WanBufRw {
    /// 网络数据包体 最大字节数
    fn set_msg_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
    }

    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, buffer: &mut Vec<u8>) -> WriteResult {
        if self.max_size < buffer.len() {
            return WriteResult::Error("msg size error".into());
        }
        let bw = &mut self.buf_writer;

        if bw.head_pos == 0 {
            fill_head(bw.next_id, buffer.len(), &mut bw.head_data);
        }
        let mut wsize = 0;
        if bw.head_pos < MSG_HEAD_SIZE {
            let result = write_data(&bw.head_data[bw.head_pos..], &mut wsize, socket);
            if result == WriteResult::Finish {
                bw.head_pos = MSG_HEAD_SIZE;
            }else {
                if WriteResult::BufferFull == result {
                    bw.head_pos += wsize;
                }
                return result
            }
        }

        let result = write_data(&buffer[bw.vec_pos..], &mut wsize, socket);
        if WriteResult::Finish == result {
            bw.head_pos = 0;
            bw.vec_pos = 0;
        }
        if WriteResult::BufferFull == result {
            bw.vec_pos += wsize;
        }

        return result;
    }

    /// 从tcp buffer中读取数据
    /// vec_share: 共享缓冲区
    fn read(&mut self, socket: &mut TcpStream, vec_share: &mut Vec<u8>) -> ReadResult<Vec<u8>> {
        let mut in_pos = 0;
        let mut vec_msg: Vec<Vec<u8>> = vec![];
        loop {
            match socket.read(&mut vec_share[in_pos..]) {
                Ok(0) => {
                    return ReadResult::Error(vec_msg, "disconnect".into());
                }
                Ok(size) => {
                    in_pos += size;

                    // 要测试一下是否可行
                    if in_pos < vec_share.len() {
                        match self.buf_reader.split_msg_data(
                            in_pos,
                            self.max_size,
                            vec_share,
                            &mut vec_msg,
                        ) {
                            None => {
                                return ReadResult::Data(vec_msg);
                            }
                            Some(err) => {
                                return ReadResult::Error(vec_msg, err);
                            }
                        }
                    } else {
                        if let Some(err) = self.buf_reader.split_msg_data(
                            in_pos,
                            self.max_size,
                            vec_share,
                            &mut vec_msg,
                        ) {
                            return ReadResult::Error(vec_msg, err);
                        }
                        in_pos = 0;
                    }
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                    match self.buf_reader.split_msg_data(
                        in_pos,
                        self.max_size,
                        vec_share,
                        &mut vec_msg,
                    ) {
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

impl BufReader {
    fn split_msg_data(
        &mut self,
        in_pos: usize,
        max_size: usize,
        vec_share: &mut Vec<u8>,
        vec_msg: &mut Vec<Vec<u8>>,
    ) -> Option<String> {
        let mut out_pos = 0;
        loop {
            if self.head_pos == 0 {
                let vec_data_len = in_pos - out_pos;
                //不够包头长度把数据存到head_data
                if vec_data_len < MSG_HEAD_SIZE {
                    self.head_pos = vec_data_len;
                    unsafe {
                        ptr::copy_nonoverlapping(
                            vec_share[out_pos..].as_ptr(),
                            self.head_data[0..].as_mut_ptr(),
                            vec_data_len,
                        );
                    }
                    return None;
                }
                self.head_pos = MSG_HEAD_SIZE;
                let head_data = &vec_share[out_pos..];
                out_pos += MSG_HEAD_SIZE;

                //获取包头数据
                let (id, msg_size) = split_head(&head_data);
                if let Some(err) = self.check_head_data(id, msg_size, max_size) {
                    return Some(err);
                }

                //分配包体内存
                self.vec_data = vec![0u8; msg_size];

                let vec_data_len = in_pos - out_pos;
                //没有包体数据
                if vec_data_len == 0 {
                    self.vec_pos = 0;
                    return None;
                }
                //不够包体数据
                if vec_data_len < msg_size {
                    self.vec_pos = vec_data_len;
                    unsafe {
                        ptr::copy_nonoverlapping(
                            vec_share[out_pos..].as_ptr(),
                            self.vec_data[0..].as_mut_ptr(),
                            vec_data_len,
                        );
                    }
                    return None;
                //足够包体数据
                } else {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            vec_share[out_pos..].as_ptr(),
                            self.vec_data[0..].as_mut_ptr(),
                            msg_size,
                        );
                    }
                    self.head_pos = 0;
                    out_pos += msg_size;
                    vec_msg.push(mem::replace(&mut self.vec_data, vec![]))
                }
            }

            if self.head_pos < MSG_HEAD_SIZE {
                //----------------------------------------------------------------------
                let vec_data_len = in_pos - out_pos;
                let head_tail_len = MSG_HEAD_SIZE - self.head_pos;
                //不够包头长度把数据存到head_data
                if vec_data_len < head_tail_len {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            vec_share[out_pos..].as_ptr(),
                            self.head_data[self.head_pos..].as_mut_ptr(),
                            vec_data_len,
                        );
                    }
                    self.head_pos += vec_data_len;
                    return None;
                }

                unsafe {
                    ptr::copy_nonoverlapping(
                        vec_share[out_pos..].as_ptr(),
                        self.head_data[self.head_pos..].as_mut_ptr(),
                        head_tail_len,
                    );
                }
                out_pos += head_tail_len;
                self.head_pos = MSG_HEAD_SIZE;
                //----------------------------------------------------------------------
                //获取包头数据
                let (id, msg_size) = split_head(&self.head_data);

                if let Some(err) = self.check_head_data(id, msg_size, max_size) {
                    return Some(err);
                }

                //分配包体内存
                self.vec_data = vec![0u8; msg_size];

                let vec_data_len = in_pos - out_pos;
                //没有包体数据
                if vec_data_len == 0 {
                    self.vec_pos = 0;
                    return None;
                }
                //不够包体数据
                if vec_data_len < msg_size {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            vec_share[out_pos..].as_ptr(),
                            self.vec_data[0..].as_mut_ptr(),
                            vec_data_len,
                        );
                    }
                    self.vec_pos = vec_data_len;
                    return None;
                //足够包体数据
                } else {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            vec_share[out_pos..].as_ptr(),
                            self.vec_data[0..].as_mut_ptr(),
                            msg_size,
                        );
                    }
                    self.head_pos = 0;
                    out_pos += msg_size;
                    vec_msg.push(mem::replace(&mut self.vec_data, vec![]))
                }
            }
            //--------------------把数据存到head_data
            let vec_data_len = in_pos - out_pos;
            let vec_data_tail = self.vec_data.len() - self.vec_pos;
            //不够包体数据
            if vec_data_len < vec_data_tail {
                unsafe {
                    ptr::copy_nonoverlapping(
                        vec_share[out_pos..].as_ptr(),
                        self.vec_data[self.vec_pos..].as_mut_ptr(),
                        vec_data_len,
                    );
                }
                self.vec_pos += vec_data_len;
                return None;
            //足够包体数据
            } else {
                unsafe {
                    ptr::copy_nonoverlapping(
                        vec_share[out_pos..].as_ptr(),
                        self.vec_data[self.vec_pos..].as_mut_ptr(),
                        vec_data_tail,
                    );
                }
                self.head_pos = 0;
                out_pos += vec_data_tail;
                vec_msg.push(mem::replace(&mut self.vec_data, vec![]))
            }
        }
    }

    #[inline]
    fn check_head_data(&mut self, id: u16, msg_size: usize, max_size: usize) -> Option<String> {
        if id != self.next_id {
            return Some("Msg Id Error".into());
        }
        if self.next_id == MSG_MAX_ID {
            self.next_id = 0;
        } else {
            self.next_id += 1;
        }

        if msg_size == 0 {
            return Some("Msg Size is 0".into());
        }
        if msg_size > max_size {
            return Some(format!("Msg Size:{} Too Large", msg_size));
        }
        return None;
    }
}
