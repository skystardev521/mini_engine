use mini_socket::tcp_buf_rw::ReadResult;
use mini_socket::tcp_buf_rw::TcpBufRw;
use mini_socket::tcp_buf_rw::WriteResult;
use std::net::TcpStream;

use crate::net_message::LanNetMsg;

pub struct LanBufRw {}

impl Default for LanBufRw {
    fn default() -> Self {
        LanBufRw {}
    }
}

impl TcpBufRw<Box<LanNetMsg>> for LanBufRw {
    /// 网络数据包体 最大字节数
    fn set_msg_max_size(&mut self, msg_max_size: usize) {}

    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, data: &Box<LanNetMsg>) -> WriteResult {
        WriteResult::Finish
    }

    /// 从tcp buffer中读取数据
    /// vec_shared: 共享缓冲区
    fn read(&mut self, socket: &mut TcpStream, vec_shared: &mut Vec<u8>) -> ReadResult<Box<LanNetMsg>> {
        ReadResult::Data(vec![])
    }
}
