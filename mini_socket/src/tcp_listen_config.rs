#[derive(Debug, Clone)]
pub struct TcpListenConfig {
    /// default: true;
    /// 是否启用 TCP_NODELAY 选项
    /// false-->缓存中积累定数据才发送
    /// true--->有数据立刻发送减少延时
    pub tcp_nodelay: bool,

    /// default:16384
    /// 消息数据最大字节
    pub msg_max_size: usize,

    /// default:10240
    /// 网络最大连接数
    pub max_tcp_socket: u32,

    /// default:512
    /// epoll触发最大事件数
    pub epoll_max_events: u16,

    /// default: 1毫秒
    /// epoll等待网络事件时长
    pub epoll_wait_timeout: i32,
    
    /// defalut: 256
    /// 待发送的最大消息数
    /// 局域网设置建议设置2048以上
    pub msg_deque_size: usize,

    /// default:0.0.0.0:9999
    pub bind_socket_addr: String,

    /// default:0
    /// 设置太少会阻塞网络通信
    /// 外网要设置大小防攻击，一般8192
    /// 局域网设置为:0 由系统分配 tcp_window_scaling = 1
    pub socket_read_buffer: u32,
    /// default:0
    /// 设置太少会阻塞网络通信
    /// 外网要设置大小防攻击，一般8192
    /// 局域网设置为:0 由系统分配 tcp_window_scaling = 1
    pub socket_write_buffer: u32,
    
}

impl TcpListenConfig {
    pub fn new() -> Self {
        TcpListenConfig {
            tcp_nodelay: true,
            msg_max_size: 16384,
            msg_deque_size: 256,
            max_tcp_socket: 10240,
            epoll_max_events: 512,
            epoll_wait_timeout: 1,
            socket_read_buffer: 0,
            socket_write_buffer: 0,
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

    pub fn set_msg_deque_size(&mut self, val: usize) -> &mut Self {
        self.msg_deque_size = val;
        self
    }
}
