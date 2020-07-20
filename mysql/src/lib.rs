mod anydata;
mod config;
mod connect;
pub mod result;

pub use anydata::test;
pub use config::Config;
pub use connect::Connect;
pub use result::QueryResult;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
