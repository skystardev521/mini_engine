use mini_socket::tcp_buf_rw::ReadResult;
use mini_socket::tcp_buf_rw::TcpBufRw;
use mini_socket::tcp_buf_rw::WriteResult;
use mini_utils::bytes;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

/// Msg Id最大值
pub const MSG_MAX_ID: u16 = 4096;

///数据包头长度4个字节
/// msg id: 0 ~ 4095
/// data size: 0 ~ (1024 * 1024)
/// |data size:13~32位|+|MID:1~12位|
pub const MSG_HEAD_SIZE: usize = 4;

pub struct WanBufRw {
    msg_max_size: usize,
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

    vec_data: Box<Vec<u8>>,
    head_data: [u8; MSG_HEAD_SIZE],
}

pub struct BufWriter {
    //包id(0~4096)
    next_id: u16,
    vec_pos: usize,
    head_pos: usize,
    vec_data: Box<Vec<u8>>,
    head_data: [u8; MSG_HEAD_SIZE],
}

impl Default for WanBufRw {
    fn default() -> Self {
        WanBufRw {
            msg_max_size: 1024,
            buf_reader: BufReader {
                next_id: 0,
                vec_pos: 0,
                head_pos: 0,
                vec_data: Box::new(vec![]),
                head_data: [0u8; MSG_HEAD_SIZE],
            },
            buf_writer: BufWriter {
                next_id: 0,
                vec_pos: 0,
                head_pos: 0,
                vec_data: Box::new(vec![]),
                head_data: [0u8; MSG_HEAD_SIZE],
            },
        }
    }
}

impl TcpBufRw<Box<Vec<u8>>> for WanBufRw {
    /// 网络数据包体 最大字节数
    fn set_msg_max_size(&mut self, msg_max_size: usize) {
        self.msg_max_size = msg_max_size;
    }

    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, data: &Box<Vec<u8>>) -> WriteResult {
        if self.msg_max_size < data.len() {
            return WriteResult::Error("msg byte size error".into());
        }
        WriteResult::Finish
    }

    /// 从tcp buffer中读取数据
    /// vec_shared: 共享缓冲区
    fn read(
        &mut self,
        socket: &mut TcpStream,
        vec_shared: &mut Vec<u8>,
    ) -> ReadResult<Box<Vec<u8>>> {
        ReadResult::Data(vec![])

        //let vec = Vec::new();
        //vec.push(Box::new(vec![0u8; 0]))

        //ReadResult::Data(Vec::new().push(Box::new(vec![])))

        //ReadResult::Data(vec![Box::new(vec![]); 0])
        /*
        let mut in_pos = 0;
        let mut result: ReadResult<Vec<u8>>;
        loop {
            match socket.read(&mut vec_shared[in_pos..]) {
                Ok(size) => {
                    if size > 0 {
                        in_pos += size;
                        if in_pos == vec_shared.len() {
                            break;
                        }
                    } else {
                        result = ReadResult::Error("disconnect".into());
                        break;
                    }
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                    result = ReadResult::BufferIsEmpty;
                    break;
                }
                Err(ref err) => {
                    result = ReadResult::Error(err.to_string());
                    break;
                }
                Err(ref err) if err.kind() == ErrorKind::Interrupted => {
                    continue; ////系统中断 再read一次
                }
            }
        }

        self.buf_reader
            .split_pack(self.msg_max_size, vec_shared, 0, in_pos);
            */
    }
}

impl BufReader {
    fn split_pack(
        &mut self,
        msg_max_size: usize,
        vec: &mut Vec<u8>,
        out_pos: usize,
        in_pos: usize,
    ) {
        let mut split_pos = out_pos;
        // bufReader中没有数据
        if self.head_pos == 0 {
            //不够包头长度
            if in_pos < MSG_HEAD_SIZE {
                self.head_pos = in_pos;
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        vec.as_ptr(),
                        self.head_data[0..in_pos].as_mut_ptr(),
                        in_pos,
                    );
                }
                return;
            }

            match self.split_head(vec, out_pos, msg_max_size) {
                Ok(msize) => {
                    self.head_pos = MSG_HEAD_SIZE;
                    self.vec_data = Box::new(vec![0u8; msize]);
                    let len = in_pos - (split_pos + MSG_HEAD_SIZE);
                    self.vec_pos = len;

                    if len == 0 {
                        return;
                    }

                    if len < msize {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                vec.as_ptr(),
                                self.vec_data[0..len].as_mut_ptr(),
                                len,
                            );
                        }
                    } else {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                vec.as_ptr(),
                                self.vec_data[0..msize].as_mut_ptr(),
                                msize,
                            );
                        }
                    }
                }
                Err(err) => {
                    return;
                }
            }
        }
    }

    fn split_head(
        &mut self,
        vec: &mut Vec<u8>,
        out_pos: usize,
        msg_max_size: usize,
    ) -> Result<usize, String> {
        let u32_val = bytes::read_u32(&vec[out_pos..]);

        //消息id
        let id = (u32_val << 20 >> 20) as u16;
        //消息字节数
        let msize = (u32_val >> 12) as usize;

        if id != self.next_id {
            return Err("Msg Id Error".into());
        }

        if msize == 0 {
            return Err("Msg Size is 0".into());
        }
        if msize > msg_max_size {
            return Err(format!("Msg Size:{} Too Large", msize));
        }

        if self.next_id == MSG_MAX_ID {
            self.next_id = 0;
        } else {
            self.next_id += 1;
        }
        Ok(msize)
    }
}

#[inline]
fn auth_head(msg_max_size: usize, mid: u16, msg_size: usize) -> Result<(), String> {
    Ok(())
}
