pub mod epoll;
pub mod message;
pub mod tcp_client;
pub mod tcp_client_config;
pub mod tcp_client_mgmt;
mod tcp_listen;
pub mod tcp_server;
pub mod tcp_server_config;
pub mod tcp_socket;
pub mod tcp_socket_const;
pub mod tcp_socket_event;
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
