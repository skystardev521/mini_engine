#[derive(Debug, Clone)]
pub struct TcpConfig {
    pub max_client: u16,
    pub msg_max_size: u32,
    pub epoll_max_events: u16,
    pub epoll_wait_timeout: i32,
    pub tcp_linsten_addr: String,
}
#[derive(Debug, Clone)]
pub struct TcpConfigBuilder {
    pub max_client: u16,
    pub msg_max_size: u32,
    pub epoll_max_events: u16,
    pub epoll_wait_timeout: i32,
    pub tcp_linsten_addr: String,
}
