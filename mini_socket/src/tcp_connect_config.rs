#[derive(Debug, Clone)]
pub struct TcpConnectConfig {
    /// 连接名
    pub name: String,
    /// 是否启用 TCP_NODELAY 选项
    /// false-->有数据立刻发送减少延时
    /// true--->缓存中积累定数据才发送
    pub tcp_nodelay: bool,
    /// 网络消息最大字节
    pub msg_max_size: usize,
    /// 要连接的socket_addr
    pub socket_addr: String,
    /// 断线重连间隔，单位毫秒
    pub reconnect_interval: u16,
    /// 待发的消息队列最大长度
    /// default: 102400
    pub msg_deque_max_len: usize,
    /// socket 的读缓冲区
    pub socket_read_buffer: u32,
    /// socket 的写缓冲区
    pub socket_write_buffer: u32,
    /// 连接超时时长，单位毫秒
    pub connect_timeout_duration: u16,
}

impl TcpConnectConfig {
    pub fn new() -> Self {
        TcpConnectConfig {
            tcp_nodelay: false,
            msg_max_size: 65536,
            reconnect_interval: 50,
            msg_deque_max_len: 102400,
            socket_read_buffer: 512000,
            socket_write_buffer: 512000,
            connect_timeout_duration: 15,
            name: "Conn_Socket_Addr".into(),
            socket_addr: "0.0.0.0:8888".into(),
        }
    }

    /// 网络消息最大字节
    pub fn set_msg_max_size(&mut self, val: usize) -> &mut Self {
        self.msg_max_size = val;
        self
    }

    pub fn set_socket_addr(&mut self, val: String) -> &mut Self {
        self.socket_addr = val.clone();
        self
    }

    /// 断线重连间隔，单位毫秒
    pub fn set_reconnect_interval(&mut self, val: u16) -> &mut Self {
        self.reconnect_interval = val;
        self
    }

    /// 是否启用 TCP_NODELAY 选项
    /// false-->有数据立刻发送减少延时
    /// true--->缓存中积累定数据才发送
    pub fn set_tcp_nodelay(&mut self, val: bool) -> &mut Self {
        self.tcp_nodelay = val;
        self
    }

    /// 等待发送的消息最大数量
    pub fn set_msg_deque_max_len(&mut self, val: usize) -> &mut Self {
        self.msg_deque_max_len = val;
        self
    }

    /// socket 的读缓冲区
    pub fn set_socket_read_buffer(&mut self, val: u32) -> &mut Self {
        self.socket_read_buffer = val;
        self
    }
    /// socket 的写缓冲区
    pub fn set_socket_write_buffer(&mut self, val: u32) -> &mut Self {
        self.socket_write_buffer = val;
        self
    }

    /// 连接超时时长，单位毫秒
    pub fn set_connect_timeout_duration(&mut self, val: u16) -> &mut Self {
        self.connect_timeout_duration = val;
        self
    }
}
