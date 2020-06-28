pub mod clients;
pub mod epoll;
pub mod message;
pub mod tcp_event;
pub mod tcp_listen;
pub mod tcp_reader;
pub mod tcp_writer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
