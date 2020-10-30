use mini_socket::tcp_listen_config::TcpListenConfig;
use mini_utils::wconfig::WConfig;

#[derive(Debug, Clone)]
pub struct Config {
    pub wconfig: WConfig,
    pub wan_listen_config: TcpListenConfig,
    pub lan_listen_config: TcpListenConfig,
}

impl Config {
    pub fn new() -> Self {
        Config {
            wconfig: WConfig::new(),
            wan_listen_config: TcpListenConfig::new(),
            lan_listen_config: TcpListenConfig::new(),
        }
    }

    pub fn read_config(&mut self, _path: &String) -> Result<(), String> {
        self.wan_listen_config
            .set_bind_socket_addr(&"0.0.0.0:9999".into());
        self.lan_listen_config
            .set_bind_socket_addr(&"0.0.0.0:6666".into());
        Ok(())
    }
}
