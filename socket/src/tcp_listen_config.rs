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
#[derive(Debug, Clone)]
pub struct TcpListenConfigBuilder {
    msg_max_size: u32,
    max_tcp_socket: u32,
    epoll_max_events: u16,
    tcp_nodelay_value: bool,
    epoll_wait_timeout: i32,
    bind_socket_addr: String,
    socket_read_buffer: u32,
    socket_write_buffer: u32,
    wait_write_msg_max_num: u16,
    single_write_msg_max_num: u16,
}

impl TcpListenConfigBuilder {
    pub fn new() -> Self {
        TcpListenConfigBuilder {
            msg_max_size: 16384,
            max_tcp_socket: 10240,
            epoll_max_events: 256,
            epoll_wait_timeout: 1,
            socket_read_buffer: 4,
            socket_write_buffer: 4,
            tcp_nodelay_value: false,
            wait_write_msg_max_num: 1280,
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

    pub fn builder(&self) -> TcpListenConfig {
        TcpListenConfig {
            msg_max_size: self.msg_max_size,
            max_tcp_socket: self.max_tcp_socket,
            tcp_nodelay_value: self.tcp_nodelay_value,
            epoll_max_events: self.epoll_max_events,
            epoll_wait_timeout: self.epoll_wait_timeout,
            socket_read_buffer: self.socket_read_buffer,
            socket_write_buffer: self.socket_write_buffer,
            bind_socket_addr: self.bind_socket_addr.clone(),
            wait_write_msg_max_num: self.wait_write_msg_max_num,
            single_write_msg_max_num: self.single_write_msg_max_num,
        }
    }
}
