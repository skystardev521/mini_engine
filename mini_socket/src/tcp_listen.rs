use crate::os_socket;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;

pub struct TcpListen {
    listen: TcpListener,
}

impl TcpListen {
    pub fn new(socket_addr: &String) -> Result<Self, String> {
        let listen = match TcpListener::bind(socket_addr) {
            Ok(listen) => listen,
            Err(err) => return Err(err.to_string()),
        };

        if let Err(err) = listen.set_nonblocking(true) {
            return Err(format!("listen.set_nonblocking{}", err));
        }

        let raw_fd = listen.as_raw_fd();

        os_socket::setsockopt(raw_fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, 1)?;
        os_socket::setsockopt(raw_fd, libc::SOL_TCP, libc::TCP_DEFER_ACCEPT, 3)?;

        Ok(TcpListen { listen })
    }

    #[inline]
    pub fn get_listen(&self) -> &TcpListener {
        &self.listen
    }
}
