use crate::tcp_buf_rw::TcpBufRw;
use crate::tcp_socket::TcpSocket;
use std::collections::HashMap;
use std::net::TcpStream;

pub struct TcpSocketMgmt<MSG> {
    //不会等于零
    next_sid: u64,
    /// 监听ID
    listen_id: u64,
    /// 每包的最大字节数
    msg_max_size: usize,
    msg_deque_max_len: usize,
    /// 可以优化使用别的数据结构
    tcp_socket_hash_map: HashMap<u64, TcpSocket<MSG>>,
}

impl<MSG> TcpSocketMgmt<MSG> {
    pub fn new(
        listen_id: u64,
        max_socket: u32,
        msg_max_size: usize,
        msg_deque_max_len: usize,
    ) -> Self {
        let tcp_socket_hash_map: HashMap<u64, TcpSocket<MSG>>;
        if max_socket < 8 {
            tcp_socket_hash_map = HashMap::with_capacity(8);
        } else {
            tcp_socket_hash_map = HashMap::with_capacity(max_socket as usize);
        }

        TcpSocketMgmt {
            listen_id,
            next_sid: 0,
            msg_max_size,
            msg_deque_max_len,
            tcp_socket_hash_map,
        }
    }

    fn next_sid(&self) -> u64 {
        let mut sid = self.next_sid;
        loop {
            sid += 1;
            if sid == 0 || sid == u64::MAX {
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
    pub fn tcp_socket_count(&self) -> u32 {
        self.tcp_socket_hash_map.len() as u32
    }

    #[inline]
    pub fn get_msg_deque_max_len(&self) -> usize {
        self.msg_deque_max_len
    }

    #[inline]
    pub fn get_tcp_socket(&mut self, sid: u64) -> Option<&mut TcpSocket<MSG>> {
        self.tcp_socket_hash_map.get_mut(&sid)
    }

    #[inline]
    pub fn del_tcp_socket(&mut self, sid: u64) -> Result<TcpSocket<MSG>, String> {
        if let Some(tcp_socket) = self.tcp_socket_hash_map.remove(&sid) {
            Ok(tcp_socket)
        } else {
            Err(format!("del_tcp_socket sid:{} not exists", sid))
        }
    }

    pub fn add_tcp_socket<TBRW>(&mut self, socket: TcpStream) -> Result<u64, String>
    where
        TBRW: TcpBufRw<MSG> + Default + 'static,
    {
        if self.tcp_socket_hash_map.len() == self.tcp_socket_hash_map.capacity() {
            return Err("Max Socket Number".into());
        }
        self.next_sid = self.next_sid();
        let mut tcp_buf_rw = Box::new(TBRW::default());
        tcp_buf_rw.set_msg_max_size(self.msg_max_size);
        self.tcp_socket_hash_map
            .insert(self.next_sid, TcpSocket::new(socket, tcp_buf_rw));
        Ok(self.next_sid)
    }
}
