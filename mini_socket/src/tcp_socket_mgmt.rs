use crate::tcp_socket::TcpSocket;
use crate::tcp_socket_rw::TcpSocketRw;
use std::collections::HashMap;
use std::net::TcpStream;

pub struct TcpSocketMgmt<MSG> {
    //不会等于零
    next_cid: u64,
    /// 监听ID
    listen_id: u64,
    /// 待发的消息队列最大长度
    msg_deque_size: usize,
    /// 可以优化使用别的数据结构
    tcp_socket_hash_map: HashMap<u64, TcpSocket<MSG>>,
}

impl<MSG> TcpSocketMgmt<MSG> {
    pub fn new(listen_id: u64, max_socket: u32, msg_deque_size: usize) -> Self {
        let tcp_socket_hash_map: HashMap<u64, TcpSocket<MSG>>;
        if max_socket < 8 {
            tcp_socket_hash_map = HashMap::with_capacity(8);
        } else {
            tcp_socket_hash_map = HashMap::with_capacity(max_socket as usize);
        }

        TcpSocketMgmt {
            listen_id,
            next_cid: 0,
            msg_deque_size,
            tcp_socket_hash_map,
        }
    }

    fn next_cid(&self) -> u64 {
        let mut cid = self.next_cid;
        loop {
            cid += 1;
            if cid == 0 || cid == u64::MAX {
                cid = 1;
            }

            if cid == self.listen_id {
                cid += 1;
            }

            if self.tcp_socket_hash_map.contains_key(&cid) {
                continue;
            }
            return cid;
        }
    }

    #[inline]
    pub fn tcp_socket_count(&self) -> u32 {
        self.tcp_socket_hash_map.len() as u32
    }

    #[inline]
    pub fn get_msg_deque_size(&self) -> usize {
        self.msg_deque_size
    }

    #[inline]
    pub fn get_tcp_socket(&mut self, cid: u64) -> Option<&mut TcpSocket<MSG>> {
        self.tcp_socket_hash_map.get_mut(&cid)
    }

    #[inline]
    pub fn del_tcp_socket(&mut self, cid: u64) -> Result<TcpSocket<MSG>, String> {
        if let Some(tcp_socket) = self.tcp_socket_hash_map.remove(&cid) {
            Ok(tcp_socket)
        } else {
            Err(format!("cid:{} not exists", cid))
        }
    }

    pub fn add_tcp_socket<TBRW>(&mut self, socket: TcpStream) -> Result<u64, String>
    where
        TBRW: TcpSocketRw<MSG> + Default + 'static,
    {
        if self.tcp_socket_hash_map.len() == self.tcp_socket_hash_map.capacity() {
            return Err("Max Socket Connect Number".into());
        }
        self.next_cid = self.next_cid();
        self.tcp_socket_hash_map.insert(
            self.next_cid,
            TcpSocket::new(socket, Box::new(TBRW::default())),
        );
        Ok(self.next_cid)
    }
}
