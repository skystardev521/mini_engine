#[derive(Debug, Clone)]
pub struct TcpListenConfig {
    pub msg_max_size: u32,
    pub max_tcp_socket: u32,
    pub epoll_max_events: u16,
    /// 是否tcp不缓存有数据就发送
    pub tcp_nodelay_value: bool,
    pub epoll_wait_timeout: i32,
    pub bind_socket_addr: String,
    pub socket_read_buffer: u32,
    pub socket_write_buffer: u32,
    pub wait_write_msg_max_num: u16,
    pub single_write_msg_max_num: u16,
}

impl TcpListenConfig {
    pub fn new() -> Self {
        TcpListenConfig {
            msg_max_size: 16384,
            max_tcp_socket: 10240,
            epoll_max_events: 256,
            epoll_wait_timeout: 1,
            socket_read_buffer: 8192,
            socket_write_buffer: 8192,
            tcp_nodelay_value: false,
            wait_write_msg_max_num: 128,
            single_write_msg_max_num: 256,
            bind_socket_addr: "0.0.0.0:9999".into(),
        }
    }

    pub fn set_max_tcp_socket(&mut self, val: u32) -> &mut Self {
        self.max_tcp_socket = val;
        self
    }

    pub fn set_msg_max_size(&mut self, val: u32) -> &mut Self {
        self.msg_max_size = val;
        self
    }

    pub fn set_epoll_max_events(&mut self, val: u16) -> &mut Self {
        self.epoll_max_events = val;
        self
    }

    pub fn set_epoll_wait_timeout(&mut self, val: i32) -> &mut Self {
        self.epoll_wait_timeout = val;
        self
    }

    pub fn set_tcp_nodelay_value(&mut self, val: bool) -> &mut Self {
        self.tcp_nodelay_value = val;
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

    pub fn set_bind_socket_addr(&mut self, val: &String) -> &mut Self {
        self.bind_socket_addr = val.clone();
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
}
