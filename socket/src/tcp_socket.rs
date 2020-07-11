use crate::message::MsgData;
use crate::tcp_socket_reader::TcpSocketReader;
use crate::tcp_socket_writer::TcpSocketWriter;
use std::net::TcpStream;

pub struct TcpSocket {
    pub epoll_events: i32,
    pub socket: TcpStream,
    pub reader: Box<TcpSocketReader>,
    pub writer: Box<TcpSocketWriter>,
}

impl TcpSocket {
    pub fn new(socket: TcpStream, max_size: u32) -> Self {
        TcpSocket {
            epoll_events: 0,
            socket: socket,
            reader: TcpSocketReader::new(max_size),
            writer: TcpSocketWriter::new(max_size),
        }
    }
}
