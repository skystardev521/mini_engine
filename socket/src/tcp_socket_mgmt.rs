use crate::tcp_socket::TcpSocket;
use crate::tcp_socket_const;
use std::collections::HashMap;
use std::net::TcpStream;

pub struct TcpSocketMgmt {
    last_id: u64,
    bind_id: u64,
    max_size: u32,
    wait_write_msg_max_num: u16,
    hash_map: HashMap<u64, TcpSocket>,
}

impl TcpSocketMgmt {
    /// max_socket: max socket number
    /// max_size: msg data max size
    pub fn new(
        bind_id: u64,
        max_socket: u32,
        max_size: u32,
        wait_write_msg_max_num: u16,
    ) -> Result<Self, String> {
        if max_socket < 8 {
            return Err("ClientNumTooSmall".into());
        }

        if max_socket > tcp_socket_const::MSG_MAX_SIZE {
            return Err("MsgSizeTooBig".into());
        }

        Ok(TcpSocketMgmt {
            last_id: 0,
            bind_id: bind_id,
            max_size: max_size,
            wait_write_msg_max_num,
            hash_map: HashMap::with_capacity(max_socket as usize),
        })
    }

    #[inline]
    pub fn total_socket(&self) -> u32 {
        self.hash_map.len() as u32
    }

    #[inline]
    pub fn get_wait_write_msg_max_num(&self) -> u16 {
        self.wait_write_msg_max_num
    }

    #[inline]
    pub fn get_tcp_socket(&mut self, id: u64) -> Option<&mut TcpSocket> {
        self.hash_map.get_mut(&id)
    }

    #[inline]
    pub fn del_socket(&mut self, id: u64) -> Result<TcpSocket, String> {
        if let Some(tcp_socket) = self.hash_map.remove(&id) {
            Ok(tcp_socket)
        } else {
            Err(format!("del_client id:{} not exists", id))
        }
    }

    pub fn new_socket(&mut self, socket: TcpStream) -> Result<u64, String> {
        if self.hash_map.len() == self.hash_map.capacity() {
            return Err("Max Socket Number".into());
        }
        loop {
            self.last_id += 1;
            if self.last_id == self.bind_id {
                self.last_id = self.bind_id + 1;
            }
            if !(self.hash_map.contains_key(&self.last_id)) {
                break;
            }
        }

        /*
        self.epoll
            .ctl_add_fd(self.last_id, socket.as_raw_fd(), libc::EPOLLIN)?;
        */
        self.hash_map
            .insert(self.last_id, TcpSocket::new(socket, self.max_size));
        Ok(self.last_id)
    }
}
