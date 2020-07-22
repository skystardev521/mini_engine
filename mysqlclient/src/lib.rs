//mod anydata;
pub mod config;
pub mod connect;
pub mod dbtask;
pub mod result;
pub mod threads;

pub use config::Config;
pub use connect::Connect;
pub use dbtask::DbTask;
pub use result::QueryResult;
pub use result::Value;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
