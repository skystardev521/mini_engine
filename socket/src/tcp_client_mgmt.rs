use crate::tcp_client::TcpClient;
use crate::tcp_client_config::TcpClientConfig;
use crate::tcp_socket::TcpSocket;
use std::net::TcpStream;

pub struct TcpClientMgmt<'a> {
    config: &'a TcpClientConfig,
    vec_tcp_client: Vec<TcpClient>,
}

impl<'a> TcpClientMgmt<'a> {
    pub fn new(config: &'a TcpClientConfig) -> Self {
        TcpClientMgmt {
            config,
            vec_tcp_client: Vec::with_capacity(config.vec_socket_addr.len()),
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        let mut vec_str: Vec<String> = vec![];
        for addr in &self.config.vec_socket_addr {
            match self.new_socket(&addr) {
                Ok(tcp_client) => {
                    self.vec_tcp_client.push(tcp_client);
                }
                Err(err) => {
                    vec_str.push(format!("connect:{} error:{}\n", addr, err));
                }
            }
        }
        if vec_str.len() == 0 {
            Ok(())
        } else {
            Err(vec_str.concat())
        }
    }

    pub fn tick(&mut self){

    }

    #[inline]
    pub fn get_tcp_socket(&mut self, id: u8) -> Option<&mut TcpClient> {
        self.vec_tcp_client.get_mut(id as usize)
    }

    #[inline]
    pub fn del_socket(&mut self, id: u8) -> Result<(), String> {
        if (id as usize) < self.vec_tcp_client.len() {
            self.vec_tcp_client.remove(id as usize);
            Ok(())
        } else {
            return Err("id >= self.vec_tcp_socket.len()".into());
        }
    }
    pub fn new_socket(&mut self, socket_addr: &String) -> Result<TcpClient, String> {
        match TcpStream::connect(socket_addr) {
            Ok(socket) => Ok(TcpClient::new(
                self.vec_tcp_client.len() as u8,
                TcpSocket::new(socket, self.config.msg_max_size),
            )),
            Err(err) => Err(format!("{}", err)),
        }
    }
}
