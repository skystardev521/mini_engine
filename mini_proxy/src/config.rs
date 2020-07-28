use mini_socket::tcp_connect_config::TcpConnectConfig;
use mini_socket::tcp_listen_config::TcpListenConfig;
use mini_utils::worker_config::WorkerConfig;

#[derive(Clone)]
pub struct Config {
    pub worker_config: WorkerConfig,
    pub tcp_listen_config: TcpListenConfig,
    pub tcp_connect_config: TcpConnectConfig,
}

impl Config {
    pub fn new() -> Self {
        Config {
            worker_config: WorkerConfig::new(),
            tcp_listen_config: TcpListenConfig::new(),
            tcp_connect_config: TcpConnectConfig::new(),
        }
    }
}
