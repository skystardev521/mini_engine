use crate::clients::Clients;

pub struct TcpEvent<'a> {
    clients: Clients,
}

impl<'a> TcpEvent<'a> {
    pub fn new(clients: Clients) -> Self {
        tcp_event { clients: clients }
    }

    fn recv(&self, id: u64) {}

    fn send(&self, id: u64) {
        //clients.hash_map.
    }

    fn error(&self, id: u64) {}

    fn accept(&self, tcp_socket: TcpStream, ip_addr: SocketAddr) {
        match tcp_socket.set_nonblocking(true) {
            Ok(()) => (),
            Err(err) => {
                match tcp_socket.shutdown(Shutdown::Both) {
                    Ok(()) => (),
                    Err(err) => println!("shutdown:{}", utils::error_kind_string(err.kind())),
                }
                println!("nonblocking:{}", utils::error_kind_string(err.kind()));
            }
        }

        println!(
            "accept id:{} fd:{},addr:{:?}",
            net_token,
            tcp_socket.as_raw_fd(),
            tcp_socket
        )
    }
}
