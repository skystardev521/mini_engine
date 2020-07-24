pub mod message;
pub mod os_epoll;
pub mod os_socket;
pub mod tcp_connect;
pub mod tcp_connect_config;
pub mod tcp_connect_mgmt;
pub mod tcp_connect_service;
pub mod tcp_listen;
pub mod tcp_listen_config;
pub mod tcp_listen_service;
pub mod tcp_socket;
pub mod tcp_socket_mgmt;
pub mod tcp_socket_reader;
pub mod tcp_socket_writer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
