use crate::tcp_socket::TcpSocket;

pub struct TcpClient {
    id: u8,
    /// 已重连次数
    connect_count: u8,
    socket_addr: String,
    last_connect_timestamp: u64,
    tcp_socket_opt: Option<TcpSocket>,
}

impl TcpClient {
    pub fn new(id: u8, socket_addr: &String, tcp_socket: Option<TcpSocket>) -> Self {
        TcpClient {
            id,
            connect_count: 0,
            last_connect_timestamp: 0,
            tcp_socket_opt: tcp_socket,
            socket_addr: socket_addr.clone(),
        }
    }
    #[inline]
    pub fn get_id(&self) -> u8 {
        self.id
    }

    #[inline]
    pub fn get_socket_addr(&self) -> &String {
        &self.socket_addr
    }

    #[inline]
    pub fn get_connect_count(&self) -> u8 {
        self.connect_count
    }
    #[inline]
    pub fn get_tcp_socket_opt(&mut self) -> &mut Option<TcpSocket> {
        &mut self.tcp_socket_opt
    }

    #[inline]
    pub fn set_tcp_socket_opt(&mut self, tcp_socket_opt: Option<TcpSocket>) {
        self.tcp_socket_opt = tcp_socket_opt
    }
    #[inline]
    pub fn get_last_connect_timestamp(&self) -> u64 {
        self.last_connect_timestamp
    }
}
