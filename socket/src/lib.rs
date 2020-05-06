pub mod entity;
pub mod epoll;
pub mod error_kind;
pub mod tcp_event;
pub mod tcp_listen;
pub mod tcp_reader;
pub mod tcp_writer;
pub mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
