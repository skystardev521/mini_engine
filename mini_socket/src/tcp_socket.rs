use crate::tcp_buf_rw::ReadResult;
use crate::tcp_buf_rw::TcpBufRw;
use crate::tcp_buf_rw::WriteResult;
use std::collections::VecDeque;
use std::mem;
use std::net::TcpStream;

pub struct TcpSocket<MSG> {
    pub epevs: i32,
    pub socket: TcpStream,
    vec_deque: VecDeque<MSG>,
    pub tcp_buf_rw: Box<dyn TcpBufRw<MSG>>,
}

impl<MSG> TcpSocket<MSG> {
    pub fn new(socket: TcpStream, tcp_buf_rw: Box<dyn TcpBufRw<MSG>>) -> Self {
        TcpSocket {
            socket,
            epevs: 0,
            tcp_buf_rw,
            vec_deque: VecDeque::new(),
        }
    }

    /// 获取当前消息列队长度
    #[inline]
    pub fn vec_queue_len(&self) -> usize {
        self.vec_deque.len()
    }

    /// 把数据存放到当前 socket 队列里
    #[inline]
    pub fn push_vec_queue(&mut self, msg: MSG) {
        self.vec_deque.push_back(msg)
    }

    /// 获取 TcpSocket 队列里的所有数据
    /// 用于断连后把数据转移到新的链接中
    #[inline]
    pub fn get_vec_queue(&mut self) -> VecDeque<MSG> {
        mem::replace(&mut self.vec_deque, VecDeque::new())
    }

    /// 把数据写到tcp buffer中
    pub fn write(&mut self) -> WriteResult {
        let socket = &mut self.socket;
        while let Some(msg) = self.vec_deque.front_mut() {
            match self.tcp_buf_rw.write(socket, msg) {
                WriteResult::Finish => {}
                WriteResult::BufferFull => return WriteResult::BufferFull,
                WriteResult::Error(err) => return WriteResult::Error(err),
            }
        }
        WriteResult::Finish
    }

    /// 从tcp buffer中读取数据
    /// vec_share: 共享缓冲区
    #[inline]
    pub fn read(&mut self, vec_share: &mut Vec<u8>) -> ReadResult<MSG> {
        let socket = &mut self.socket;
        self.tcp_buf_rw.read(socket, vec_share)
    }
}
