use crate::tcp_connect_config::TcpConnectConfig;
use crate::tcp_socket::TcpSocket;

pub struct TcpConnect {
    id: u64,
    config: TcpConnectConfig,
    last_reconnect_timestamp: u64,
    tcp_socket_opt: Option<TcpSocket>,
}

impl TcpConnect {
    pub fn new(id: u64, config: TcpConnectConfig, tcp_socket: Option<TcpSocket>) -> Self {
        TcpConnect {
            id,
            config,
            tcp_socket_opt: tcp_socket,
            last_reconnect_timestamp: 0,
        }
    }
    #[inline]
    pub fn get_id(&self) -> u64 {
        self.id
    }

    #[inline]
    pub fn get_config(&self) -> &TcpConnectConfig {
        &self.config
    }

    /// 最后重连时间戳
    #[inline]
    pub fn get_last_reconnect_timestamp(&self) -> u64 {
        self.last_reconnect_timestamp
    }

    #[inline]
    pub fn get_tcp_socket_opt(&mut self) -> &mut Option<TcpSocket> {
        &mut self.tcp_socket_opt
    }

    #[inline]
    pub fn set_tcp_socket_opt(&mut self, tcp_socket_opt: Option<TcpSocket>) {
        self.tcp_socket_opt = tcp_socket_opt
    }
}
