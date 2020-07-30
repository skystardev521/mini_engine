use mini_socket::tcp_connect_config::TcpConnectConfig;
use mini_socket::tcp_listen_config::TcpListenConfig;
use mini_utils::worker_config::WorkerConfig;

#[derive(Debug, Clone)]
pub struct Config {
    pub worker_config: WorkerConfig,
    pub wan_listen_config: TcpListenConfig,
    pub lan_listen_config: TcpListenConfig,
}

impl Config {
    pub fn new() -> Self {
        Config {
            worker_config: WorkerConfig::new(),
            wan_listen_config: TcpListenConfig::new(),
            lan_listen_config: TcpListenConfig::new(),
        }
    }

    pub fn read_config(&mut self, path: &String) -> Result<(), String> {
        self.wan_listen_config
            .set_bind_socket_addr(&"0.0.0.0:9999".into());
        self.lan_listen_config
            .set_bind_socket_addr(&"0.0.0.0:6666".into());
        Ok(())
    }
}
