use mini_socket::tcp_buf_rw::ReadResult;
use mini_socket::tcp_buf_rw::TcpBufRw;
use mini_socket::tcp_buf_rw::WriteResult;
use std::net::TcpStream;

use crate::net_message::NetMsg;

pub struct NetBufRw {
    t: u16,
}

impl Default for NetBufRw {
    fn default() -> Self {
        NetBufRw { t: 0 }
    }
}

impl TcpBufRw<NetMsg> for NetBufRw {
    /// 网络数据包体 最大字节数
    fn set_max_size(&mut self, size: usize) {}

    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, data: &NetMsg) -> WriteResult {
        WriteResult::Finish
    }

    /// 从tcp buffer中读取数据
    /// vec_shared: 共享缓冲区
    fn read(&mut self, socket: &mut TcpStream, vec_shared: &mut Vec<u8>) -> ReadResult<NetMsg> {
        ReadResult::BufferIsEmpty
    }
}
