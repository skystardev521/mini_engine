#[derive(Debug, Clone)]
pub struct TcpConnectConfig {
    /// 网络消息最大字节
    pub msg_max_size: u32,
    /// os_epoll 触发最大事件数
    pub epoll_max_events: u16,
    /// 是否tcp不缓存有数据就发送
    pub tcp_nodelay_value: bool,
    /// socket 的读缓冲区
    pub socket_read_buffer: u32,
    /// socket 的写缓冲区
    pub socket_write_buffer: u32,
    /// 等待发送的消息最大数量
    pub wait_write_msg_max_num: u16,
    /// 每次获取发送到网络中的最大消息数量
    pub single_write_msg_max_num: u16,
    /// 要创建连接的socket addr 集合
    pub vec_socket_addr: Vec<String>,
    /// 连接超时时长，单位毫秒
    pub connect_timeout_duration: u16,
    /// 重连间隔，单位毫秒
    pub reconnect_socket_interval: u16,
}

impl TcpConnectConfig {
    pub fn new() -> Self {
        TcpConnectConfig {
            msg_max_size: 1024 * 32,
            epoll_max_events: 256,
            tcp_nodelay_value: false,
            socket_read_buffer: 65536,
            socket_write_buffer: 65536,
            wait_write_msg_max_num: 1024,
            single_write_msg_max_num: 512,
            vec_socket_addr: Vec::new(),
            connect_timeout_duration: 10,
            reconnect_socket_interval: 50,
        }
    }

    pub fn set_msg_max_size(&mut self, val: u32) -> &mut Self {
        self.msg_max_size = val;
        self
    }
    pub fn set_epoll_max_events(&mut self, val: u16) -> &mut Self {
        self.epoll_max_events = val;
        self
    }
    pub fn set_socket_read_buffer(&mut self, val: u32) -> &mut Self {
        self.socket_read_buffer = val;
        self
    }
    pub fn set_socket_write_buffer(&mut self, val: u32) -> &mut Self {
        self.socket_write_buffer = val;
        self
    }

    pub fn set_tcp_nodelay_value(&mut self, val: bool) -> &mut Self {
        self.tcp_nodelay_value = val;
        self
    }

    pub fn set_wait_write_msg_max_num(&mut self, val: u16) -> &mut Self {
        self.wait_write_msg_max_num = val;
        self
    }

    pub fn set_single_write_msg_max_num(&mut self, val: u16) -> &mut Self {
        self.single_write_msg_max_num = val;
        self
    }

    pub fn set_vec_socket_addr(&mut self, val: &Vec<String>) -> &mut Self {
        self.vec_socket_addr = val.clone();
        self
    }
    pub fn set_connect_timeout_duration(&mut self, val: u16) -> &mut Self {
        self.connect_timeout_duration = val;
        self
    }
    pub fn set_reconnect_socket_interval(&mut self, val: u16) -> &mut Self {
        self.reconnect_socket_interval = val;
        self
    }
}
