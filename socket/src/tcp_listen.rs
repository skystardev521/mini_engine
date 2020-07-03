use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use utils::native;

pub struct TcpListen {
    listen: TcpListener,
}

impl TcpListen {
    pub fn new(socket_addr: &String) -> Result<Self, String> {
        let listen = match TcpListener::bind(socket_addr) {
            Ok(listen) => listen,
            Err(err) => return Err(format!("{}", err)),
        };

        match listen.set_nonblocking(true) {
            Ok(()) => (),
            Err(err) => return Err(format!("{}", err)),
        }

        match native::setsockopt(listen.as_raw_fd(), libc::SO_REUSEADDR, 1) {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        Ok(TcpListen { listen })
    }

    #[inline]
    pub fn get_listen(&self) -> &TcpListener {
        &self.listen
    }
}
