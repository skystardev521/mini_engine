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

impl TcpBufRw<LanNetMsg> for LanBufRw {
    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, data: &mut LanNetMsg) -> WriteResult {
        WriteResult::Finish
    }

    /// 从tcp buffer中读取数据
    /// vec_share: 共享缓冲区
    fn read(&mut self, socket: &mut TcpStream, vec_share: &mut Vec<u8>) -> ReadResult<LanNetMsg> {
        ReadResult::Data(vec![])
    }
}
