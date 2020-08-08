#[derive(Debug, Clone)]
pub struct TcpListenConfig {
    /// 是否启用 TCP_NODELAY 选项
    /// false-->有数据立刻发送减少延时
    /// true--->缓存中积累定数据才发送
    /// default: false;
    pub tcp_nodelay: bool,
    /// default:16384
    pub msg_max_size: usize,

    /// default:10240
    pub max_tcp_socket: u32,
    /// epoll触发最大事件数
    /// default:512
    pub epoll_max_events: u16,

    /// default: 1
    pub epoll_wait_timeout: i32,

    /// 等待发送的最大消息数
    /// defalut: 256
    pub msg_deque_max_len: usize,

    /// default:0.0.0.0:9999
    pub bind_socket_addr: String,
    /// default:8192
    pub socket_read_buffer: u32,
    /// default:8192
    pub socket_write_buffer: u32,

    /// 单次发送消息到tcp_listen的数量
    pub single_write_msg_max_num: u16,
    /// 单次调用epoll_wait函数最大次数
    pub single_call_epoll_wait_max_num: u8,
}

impl TcpListenConfig {
    pub fn new() -> Self {
        TcpListenConfig {
            tcp_nodelay: false,
            msg_max_size: 16384,
            msg_deque_max_len: 256,
            max_tcp_socket: 10240,
            epoll_max_events: 512,
            epoll_wait_timeout: 1,
            socket_read_buffer: 8192,
            socket_write_buffer: 8192,

            single_write_msg_max_num: 2048,
            single_call_epoll_wait_max_num: 16,
            bind_socket_addr: "0.0.0.0:9999".into(),
        }
    }

    pub fn set_max_tcp_socket(&mut self, val: u32) -> &mut Self {
        self.max_tcp_socket = val;
        self
    }

    pub fn set_msg_max_size(&mut self, val: usize) -> &mut Self {
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

    pub fn set_tcp_nodelay(&mut self, val: bool) -> &mut Self {
        self.tcp_nodelay = val;
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

    pub fn set_msg_deque_max_len(&mut self, val: usize) -> &mut Self {
        self.msg_deque_max_len = val;
        self
    }

    pub fn set_single_write_msg_max_num(&mut self, val: u16) -> &mut Self {
        self.single_write_msg_max_num = val;
        self
    }
    pub fn set_single_call_epoll_wait_max_num(&mut self, val: u8) -> &mut Self {
        self.single_call_epoll_wait_max_num = val;
        self
    }
}
