#[derive(Debug, Clone)]
pub struct TcpServerConfig {
    pub max_socket: u32,
    pub msg_max_size: u32,
    pub epoll_max_events: u16,
    pub epoll_wait_timeout: i32,
    pub bind_socket_addr: String,
    pub socket_read_buffer: u32,
    pub socket_write_buffer: u32,
    pub wait_write_msg_max_num: u16,
}
#[derive(Debug, Clone)]
pub struct TcpServerConfigBuilder {
    pub max_socket: u32,
    pub msg_max_size: u32,
    pub epoll_max_events: u16,
    pub epoll_wait_timeout: i32,
    pub bind_socket_addr: String,
    pub socket_read_buffer: u32,
    pub socket_write_buffer: u32,
    pub wait_write_msg_max_num: u16,
}

impl TcpServerConfig {}

impl TcpServerConfigBuilder {
    pub fn new() -> TcpServerConfigBuilder {
        TcpServerConfigBuilder {
            max_socket: 1024,
            msg_max_size: 32,
            epoll_max_events: 256,
            epoll_wait_timeout: 1,
            socket_read_buffer: 4,
            socket_write_buffer: 4,
            wait_write_msg_max_num: 128,
            bind_socket_addr: "0.0.0.0:9999".into(),
        }
    }

    pub fn set_max_socket(&mut self, val: u32) -> &mut TcpServerConfigBuilder {
        self.max_socket = val;
        self
    }

    pub fn set_msg_max_size(&mut self, val: u32) -> &mut TcpServerConfigBuilder {
        self.max_socket = val;
        self
    }

    pub fn set_epoll_max_events(&mut self, val: u16) -> &mut TcpServerConfigBuilder {
        self.epoll_max_events = val;
        self
    }

    pub fn set_epoll_wait_timeout(&mut self, val: i32) -> &mut TcpServerConfigBuilder {
        self.epoll_wait_timeout = val;
        self
    }

    pub fn set_socket_read_buffer(&mut self, val: u32) -> &mut TcpServerConfigBuilder {
        self.socket_read_buffer = val;
        self
    }

    pub fn set_socket_write_buffer(&mut self, val: u32) -> &mut TcpServerConfigBuilder {
        self.socket_write_buffer = val;
        self
    }

    pub fn set_bind_socket_addr(&mut self, val: &String) -> &mut TcpServerConfigBuilder {
        self.bind_socket_addr = val.clone();
        self
    }

    pub fn set_wait_write_msg_max_num(&mut self, val: u16) -> &mut TcpServerConfigBuilder {
        self.wait_write_msg_max_num = val;
        self
    }

    pub fn builder(&self) -> TcpServerConfig {
        TcpServerConfig {
            max_socket: self.max_socket,
            msg_max_size: self.msg_max_size,
            epoll_max_events: self.epoll_max_events,
            epoll_wait_timeout: self.epoll_wait_timeout,
            socket_read_buffer: self.socket_read_buffer,
            socket_write_buffer: self.socket_write_buffer,
            bind_socket_addr: self.bind_socket_addr.clone(),
            wait_write_msg_max_num: self.wait_write_msg_max_num,
        }
    }
}
