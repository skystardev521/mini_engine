use crate::message;
use crate::tcp_socket::TcpSocket;
use std::collections::HashMap;
use std::net::TcpStream;

pub struct TcpSocketMgmt {
    //不会等于零
    next_sid: u64,
    listen_id: u64,
    msg_max_size: u32,
    wait_write_msg_max_num: u16,
    /// 可以优化使用别的数据结构
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
        if max_socket < 1 {
            return Err("socket Too Small".into());
        }

        if msg_max_size > message::MSG_MAX_SIZE {
            return Err("msg size too big".into());
        }

        Ok(TcpSocketMgmt {
            next_sid: 0,
            listen_id: listen_id,
            msg_max_size: msg_max_size,
            wait_write_msg_max_num: wait_write_msg_max_num,
            tcp_socket_hash_map: HashMap::with_capacity(max_socket as usize),
        })
    }

    fn next_sid(&self) -> u64 {
        let mut sid = self.next_sid;
        loop {
            sid += 1;
            if sid == 0 {
                sid = 1;
            }
            if sid == self.listen_id {
                sid += 1;
            }
            if self.tcp_socket_hash_map.contains_key(&sid) {
                continue;
            }
            return sid;
        }
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
    pub fn get_tcp_socket(&mut self, sid: u64) -> Option<&mut TcpSocket> {
        self.tcp_socket_hash_map.get_mut(&sid)
    }

    #[inline]
    pub fn del_tcp_socket(&mut self, sid: u64) -> Result<TcpSocket, String> {
        if let Some(tcp_socket) = self.tcp_socket_hash_map.remove(&sid) {
            Ok(tcp_socket)
        } else {
            Err(format!("del_client id:{} not exists", sid))
        }
    }

    pub fn add_tcp_socket(&mut self, socket: TcpStream) -> Result<u64, String> {
        if self.tcp_socket_hash_map.len() == self.tcp_socket_hash_map.capacity() {
            return Err("Max Socket Number".into());
        }
        self.next_sid = self.next_sid();
        self.tcp_socket_hash_map
            .insert(self.next_sid, TcpSocket::new(socket, self.msg_max_size));
        Ok(self.next_sid)
    }
}
