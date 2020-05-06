use crate::epoll::Epoll;
use std::collections::HashMap;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::rc::Rc;

use crate::entity{
    NetData,
};
use crate::tcpreader::TcpReader;
use crate::tcpwriter::TcpWriter;

pub struct Session {
    stream: TcpStream,
    tcpreader: TcpReader,
    tcpwriter: TcpWriter,
    //socket_addr: SocketAddr,  TcpStream.peer_addr(&self) -> Result<SocketAddr>
}

pub struct SessionMgmt {
    //count: usize,
    Sessions: Box<HashMap<u64, Session>>,
}

impl SessionMgmt {
    pub fn new(session_count: usize) -> Self {
        SessionMgmt {
            //count: session_count,
            Sessions: Box::new(HashMap::with_capacity(session_count)),
        }
    }
    pub fn get_count(&self) -> usize {
        self.Sessions.len()
    }

    pub fn add_session(
        &mut self,
        id: u64,
        stream: TcpStream,
    ) -> Option<Session> {
        if self.Sessions.len() == self.Sessions.capacity() {
            return None;
        }
        self.Sessions.insert(id, Session::new(stream))
    }
}
impl Session {
    fn new(stream: TcpStream, buffer_size: u32, pack_max_size, pack_cb: fn(&[u8])) -> Self {
        Session {
            stream: stream,
            tcpwriter: TcpWriter::new(),
            tcpreader: TcpReader::new(buffer_size, pack_max_size, pack_cb),
        }
    }
}
