use mini_socket::tcp_connect_config::TcpConnectConfig;
use mini_utils::worker_config::WorkerConfig;

#[derive(Debug, Clone)]
pub struct Config {
    pub worker_config: WorkerConfig,
    pub vec_tcp_connect_config: Vec<TcpConnectConfig>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            worker_config: WorkerConfig::new(),
            vec_tcp_connect_config: Vec::new(),
        }
    }

    pub fn read_config(&mut self, path: &String) -> Result<(), String> {
        for _ in 0..100 {
            let mut connect_config = TcpConnectConfig::new();
            connect_config.set_socket_addr("0.0.0.0:6666".into());
            self.vec_tcp_connect_config.push(connect_config);
        }

        Ok(())
    }
}
