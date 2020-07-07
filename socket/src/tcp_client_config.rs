//管理Tcp连接
pub struct TcpClientConfig {
    pub msg_max_size: u32,
    pub socket_read_buffer: u32,
    pub socket_write_buffer: u32,
    pub wait_write_msg_max_num: u16,
    pub vec_socket_addr: Vec<String>,
}
