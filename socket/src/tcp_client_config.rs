pub struct TcpClientConfig {
    pub msg_max_size: u32,
    pub socket_read_buffer: u32,
    pub socket_write_buffer: u32,
    pub wait_write_msg_max_num: u16,
    pub vec_socket_addr: Vec<String>,
    /// 单位毫秒
    pub connect_timeout_duration: u16,
    /// 单位毫秒
    pub reconnect_socket_interval: u16,
}
