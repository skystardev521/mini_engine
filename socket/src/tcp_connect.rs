use crate::tcp_socket::TcpSocket;

pub struct TcpConnect {
    id: u64,
    /// 已重连次数
    reconnect_count: u8,
    socket_addr: String,
    last_reconnect_timestamp: u64,
    tcp_socket_opt: Option<TcpSocket>,
}

impl TcpConnect {
    pub fn new(id: u64, socket_addr: &String, tcp_socket: Option<TcpSocket>) -> Self {
        TcpConnect {
            id,
            reconnect_count: 0,
            last_reconnect_timestamp: 0,
            tcp_socket_opt: tcp_socket,
            socket_addr: socket_addr.clone(),
        }
    }
    #[inline]
    pub fn get_id(&self) -> u64 {
        self.id
    }

    #[inline]
    pub fn get_socket_addr(&self) -> &String {
        &self.socket_addr
    }

    #[inline]
    pub fn get_reconnect_count(&self) -> u8 {
        self.reconnect_count
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
    pub fn get_last_reconnect_timestamp(&self) -> u64 {
        self.last_reconnect_timestamp
    }
}
