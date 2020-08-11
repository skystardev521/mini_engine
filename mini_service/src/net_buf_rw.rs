use mini_socket::tcp_buf_rw::ReadResult;
use mini_socket::tcp_buf_rw::TcpBufRw;
use mini_socket::tcp_buf_rw::WriteResult;
use std::net::TcpStream;

use crate::net_message::NetMsg;

///数据包头长度4个字节
/// msg id: 0 ~ 4095
/// data size: 0 ~ (1024 * 1024)
///|data size:13~32位|+|MID:1~12位|
pub const MSG_HEAD_SIZE: usize = 4;

pub struct NetBufRw {
    msg_max_size: usize,
}

pub struct BufReader {
    //包id(0~4096)
    next_mid: u16,
    max_size: usize,
    head_pos: usize,
    data_pos: usize,
    msg_data: Vec<u8>,
    head_data: [u8; MSG_HEAD_SIZE],
}

pub struct BufWriter {
    //包id(0~4096)
    next_mid: u16,
    max_size: usize,
    head_pos: usize,
    data_pos: usize,
    msg_data: Vec<u8>,
    head_data: [u8; MSG_HEAD_SIZE],
}

impl Default for NetBufRw {
    fn default() -> Self {
        NetBufRw { msg_max_size: 1024 }
    }
}

impl TcpBufRw<Box<NetMsg>> for NetBufRw {
    /// 网络数据包体 最大字节数
    fn set_msg_max_size(&mut self, msg_max_size: usize) {
        self.msg_max_size = msg_max_size;
    }

    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, data: &Box<NetMsg>) -> WriteResult {
        WriteResult::Finish
    }

    /// 从tcp buffer中读取数据
    /// vec_shared: 共享缓冲区
    fn read(
        &mut self,
        socket: &mut TcpStream,
        vec_shared: &mut Vec<u8>,
    ) -> ReadResult<Box<NetMsg>> {
        //let vec_msg_box:Vec<Box<NetMsg>> = Vec::new();
        ReadResult::Data(vec![])
    }
}
