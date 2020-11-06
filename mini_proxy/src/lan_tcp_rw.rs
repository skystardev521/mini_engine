use mini_socket::tcp_socket_rw::ReadResult;
use mini_socket::tcp_socket_rw::TcpSocketRw;
use mini_socket::tcp_socket_rw::WriteResult;
use mini_utils::bytes;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

use mini_socket::tcp_socket_msg::MsgData;

/// Msg Id最大值
pub const MSG_MAX_ID: u16 = 4095;

///数据包头长度18个字节
/// msg id: 0 ~ 4095
/// msg size: 0 ~ (1024 * 1024)
/// |msg size:13~32位|+|mid:1~12位|
/// |(msg size + msg id):32|pid:16|ext:32|uid:64|
pub const MSG_HEAD_SIZE: usize = 18;

/// 数据包体最大字节数
pub const MSG_MAX_SIZE: usize = 1024 * 1024;

macro_rules! min_val {
    ($v1:expr, $v2:expr) => {
        if $v1 < $v2 {
            $v1
        } else {
            $v2
        }
    };
}

macro_rules! next_msg_id {
    ($id:expr) => {
        if $id == MSG_MAX_ID {
            0
        } else {
            $id + 1
        }
    };
}

macro_rules! copy_data {
    ($src:expr,$dst:expr,$count:expr) => {
        unsafe {
            std::ptr::copy_nonoverlapping($src.as_ptr(), $dst.as_mut_ptr(), $count);
        }
    };
}

//id: u16, msize: usize, msg: &NetMsg, buffer: &mut [u8]
macro_rules! fill_head_data {
    ($id:expr, $buf:expr, $msg:expr) => {
        let msize = $msg.buf.len() as u32;
        let u32_val = (msize << 12) + $id as u32;
        bytes::write_u32($buf, u32_val);
        bytes::write_u16(&mut $buf[4..], $msg.pid);
        bytes::write_u32(&mut $buf[6..], $msg.ext);
        bytes::write_u64(&mut $buf[10..], $msg.uid);
    };
}

macro_rules! read_head_u32 {
    ($buf:expr) => {
        bytes::read_u32($buf);
    };
}

macro_rules! head_sign_data {
    ($u32_val:expr) => {
        //消息id                       //消息字节
        (($u32_val << 20 >> 20) as u16, ($u32_val >> 12) as usize)
    };
}

macro_rules! read_head_data {
    ($buf:expr) => {
        MsgData {
            buf: vec![],
            pid: bytes::read_u16(&$buf[4..]),
            ext: bytes::read_u32(&$buf[6..]),
            uid: bytes::read_u64(&$buf[10..]),
        }
    };
}

pub struct LanTcpRw {
    buf_reader: BufReader,
    buf_writer: BufWriter,
}

pub struct BufReader {
    //包id(0~4096)
    id: u16,
    /// 0:no data
    head_pos: usize,
    /// 0:no data
    body_pos: usize,
    body_data: Vec<u8>,
    head_data: [u8; MSG_HEAD_SIZE],
}

pub struct BufWriter {
    //包id(0~4096)
    id: u16,
    body_pos: usize,
    head_pos: usize,
    /// 是否已填充head
    head_is_fill: bool,
    head_data: [u8; MSG_HEAD_SIZE],
}

impl Default for LanTcpRw {
    fn default() -> Self {
        LanTcpRw {
            buf_reader: BufReader {
                id: 0,
                body_pos: 0,
                head_pos: 0,
                body_data: vec![],
                head_data: [0u8; MSG_HEAD_SIZE],
            },
            buf_writer: BufWriter {
                id: 0,
                body_pos: 0,
                head_pos: 0,
                head_is_fill: false,
                head_data: [0u8; MSG_HEAD_SIZE],
            },
        }
    }
}

impl LanTcpRw {
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
}

impl TcpSocketRw<MsgData> for LanTcpRw {
    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, msg: &mut MsgData) -> WriteResult {
        if MSG_MAX_SIZE < msg.buf.len() {
            return WriteResult::Error(format!("msg size too large:{}", msg.buf.len()));
        }
        let bw = &mut self.buf_writer;

        // 新的消息包
        if bw.head_pos == 0 && !bw.head_is_fill {
            bw.head_is_fill = true;
            let bw_id = bw.id;
            bw.id = next_msg_id!(bw.id);
            fill_head_data!(bw_id, &mut bw.head_data, msg);
        }

        // 写头部数据
        if bw.head_pos < MSG_HEAD_SIZE {
            // 写成功的字节数
            let mut wsize = 0;
            let result = Self::write_data(&bw.head_data[bw.head_pos..], &mut wsize, socket);
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
        // 把包体数据写入
        let result = Self::write_data(&msg.buf[bw.body_pos..], &mut wsize, socket);
        if WriteResult::Finish == result {
            bw.head_pos = 0;
            bw.body_pos = 0;
            bw.head_is_fill = false;
        }
        if WriteResult::BufferFull == result {
            bw.body_pos += wsize;
        }
        return result;
    }

    /// 从tcp bufferfer中读取数据
    /// buffer: 共享缓冲区 这方式用于读小包的方案
    fn read(&mut self, socket: &mut TcpStream, share_buffer: &mut Vec<u8>) -> ReadResult<MsgData> {
        let mut in_pos = 0;
        let mut vec_msg: Vec<MsgData> = vec![];
        let br = &mut self.buf_reader;

        loop {
            match socket.read(&mut share_buffer[in_pos..]) {
                Ok(0) => {
                    return ReadResult::Error(vec_msg, "disconnect".into());
                }
                Ok(size) => {
                    in_pos += size;
                    
                    // 分解数据包
                    if let Some(err) = br.split_data(in_pos, share_buffer, &mut vec_msg) {
                        return ReadResult::Error(vec_msg, err);
                    }
                    // 读完了TCP缓存区数据
                    if in_pos < share_buffer.capacity() {
                        return ReadResult::Data(vec_msg);
                    }

                    in_pos = 0; // 重新开始读到buffer中
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                    match br.split_data(in_pos, share_buffer, &mut vec_msg) {
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
    fn split_data(
        &mut self,
        in_pos: usize,
        buffer: &Vec<u8>,
        vec_msg: &mut Vec<MsgData>,
    ) -> Option<String> {
        let mut out_pos = 0;
        loop {
            if self.head_pos < MSG_HEAD_SIZE {
                //----------------------------------------------------------------------
                let data_len = in_pos - out_pos;
                let tail_len = MSG_HEAD_SIZE - self.head_pos;

                let min_len = min_val!(data_len, tail_len);

                copy_data!(buffer[out_pos..], self.head_data[self.head_pos..], min_len);

                self.head_pos += min_len;

                //数据不够包头长度
                if min_len < tail_len {
                    return None;
                }

                out_pos += tail_len;
                
                //数据够包头长度，获取包头数据
                let (mid, msize) = head_sign_data!(read_head_u32!(&self.head_data));

                if let Some(err) = self.check_sign_data(mid, msize) {
                    return Some(err);
                }

                //包体没有数据
                if msize == 0{
                    self.head_pos = 0;
                    //没有包体的消息
                    vec_msg.push(read_head_data!(&self.head_data));
                    continue;
                }else{
                    //分配包体内存
                    self.body_pos = 0;
                    self.body_data = vec![0u8; msize];
                }
            };

            let data_len = in_pos - out_pos;
            let tail_len = self.body_data.capacity() - self.body_pos;

            let min_len = min_val!(data_len, tail_len);

            copy_data!(buffer[out_pos..], self.body_data[self.body_pos..], min_len);

            //不够包体所以需数据
            if data_len < tail_len {
                self.body_pos += min_len;
                return None;
            }else{
                self.head_pos = 0;
                out_pos += min_len;
                let mut msg = read_head_data!(&self.head_data);
                msg.buf = std::mem::replace(&mut self.body_data, vec![]);
                vec_msg.push(msg); // 分割了一个完整的包
            }
        }
    }

    /// 检查包id 及 包字节
    #[inline]
    fn check_sign_data(&mut self, id: u16, msg_size: usize) -> Option<String> {
        if id != self.id {
            return Some("Msg Id does not match".into());
        }
        self.id = next_msg_id!(id);

        if msg_size > MSG_MAX_SIZE {
            return Some(format!("Msg Size:{} Too Large", msg_size));
        }
        return None;
    }
}