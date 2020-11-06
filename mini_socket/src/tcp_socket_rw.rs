use std::net::TcpStream;

#[derive(PartialEq)]
pub enum WriteResult {
    Finish,
    BufferFull,
    Error(String),
}

pub enum ReadResult<MSG> {
    Data(Vec<MSG>),
    Error(Vec<MSG>, String),
}

pub trait TcpSocketRw<MSG> {
    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, data: &mut MSG) -> WriteResult;

    /// 从tcp buffer中读取数据
    /// share_buffer: 共享缓冲区
    fn read(&mut self, socket: &mut TcpStream, share_buffer: &mut Vec<u8>) -> ReadResult<MSG>;
}
