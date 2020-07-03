use crate::tcp_socket::TcpSocket;
use crate::tcp_socket_const;
use std::collections::HashMap;
use std::net::TcpStream;

pub struct TcpSocketMgmt {
    next_id: u64,
    listen_id: u64,
    msg_max_size: u32,
    wait_write_msg_max_num: u16,
    tcp_socket_hash_map: HashMap<u64, TcpSocket>,
}

impl TcpSocketMgmt {
    /// max_socket: max socket number
    /// msg_max_size: msg data max size
    pub fn new(
        listen_id: u64,
        max_socket: u32,
        msg_max_size: u32,
        wait_write_msg_max_num: u16,
    ) -> Result<Self, String> {
        if max_socket < 8 {
            return Err("socket Too Small".into());
        }

        if max_socket > tcp_socket_const::MSG_MAX_SIZE {
            return Err("msg size too big".into());
        }

        Ok(TcpSocketMgmt {
            next_id: 0,
            listen_id: listen_id,
            msg_max_size: msg_max_size,
            wait_write_msg_max_num: wait_write_msg_max_num,
            tcp_socket_hash_map: HashMap::with_capacity(max_socket as usize),
        })
    }

    #[inline]
    pub fn total_socket(&self) -> u32 {
        self.tcp_socket_hash_map.len() as u32
    }

    #[inline]
    pub fn get_wait_write_msg_max_num(&self) -> u16 {
        self.wait_write_msg_max_num
    }

    #[inline]
    pub fn get_tcp_socket(&mut self, id: u64) -> Option<&mut TcpSocket> {
        self.tcp_socket_hash_map.get_mut(&id)
    }

    #[inline]
    pub fn del_socket(&mut self, id: u64) -> Result<TcpSocket, String> {
        if let Some(tcp_socket) = self.tcp_socket_hash_map.remove(&id) {
            Ok(tcp_socket)
        } else {
            Err(format!("del_client id:{} not exists", id))
        }
    }

    pub fn new_socket(&mut self, socket: TcpStream) -> Result<u64, String> {
        if self.tcp_socket_hash_map.len() == self.tcp_socket_hash_map.capacity() {
            return Err("Max Socket Number".into());
        }
        loop {
            self.next_id += 1;
            if self.next_id == self.listen_id {
                self.next_id = self.listen_id + 1;
            }
            if !(self.tcp_socket_hash_map.contains_key(&self.next_id)) {
                break;
            }
        }
        self.tcp_socket_hash_map
            .insert(self.next_id, TcpSocket::new(socket, self.msg_max_size));
        Ok(self.next_id)
    }
}
