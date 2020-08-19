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

pub trait TcpBufRw<MSG> {
    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, data: &mut MSG) -> WriteResult;

    /// 从tcp buffer中读取数据
    /// vec_share: 共享缓冲区
    fn read(&mut self, socket: &mut TcpStream, vec_share: &mut Vec<u8>) -> ReadResult<MSG>;
}
