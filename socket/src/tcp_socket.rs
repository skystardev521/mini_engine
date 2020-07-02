use crate::message::MsgData;
use crate::tcp_socket_reader::TcpSocketReader;
use crate::tcp_socket_writer::TcpSocketWriter;
use std::net::TcpStream;

pub enum ReadResult {
    Error(String),
    ReadZeroSize,
    BufferIsEmpty,
    Data(Box<MsgData>),
}

#[derive(Debug)]
pub enum WriteResult {
    Finish,
    BufferFull,
    Error(String),
}

pub struct TcpSocket {
    pub socket: TcpStream,
    pub reader: Box<TcpSocketReader>,
    pub writer: Box<TcpSocketWriter>,
}

impl TcpSocket {
    pub fn new(socket: TcpStream, max_size: u32) -> Self {
        TcpSocket {
            socket: socket,
            reader: TcpSocketReader::new(max_size),
            writer: TcpSocketWriter::new(max_size),
        }
    }
}
