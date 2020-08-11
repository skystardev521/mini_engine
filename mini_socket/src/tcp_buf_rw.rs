use std::net::TcpStream;

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
    /// 网络数据包体 最大字节数
    fn set_msg_max_size(&mut self, msg_max_size: usize);

    /// 把数据写到tcp buffer中
    fn write(&mut self, socket: &mut TcpStream, data: &MSG) -> WriteResult;

    /// 从tcp buffer中读取数据
    /// vec_shared: 共享缓冲区
    fn read(&mut self, socket: &mut TcpStream, vec_shared: &mut Vec<u8>) ->ReadResult<MSG>;
}
