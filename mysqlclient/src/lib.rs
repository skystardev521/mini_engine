//mod anydata;
pub mod config;
pub mod connect;
pub mod message;
pub mod result;
pub mod task;
pub mod test;
pub mod threads;

pub use config::ConnConfig;
pub use config::ThreadConfig;
pub use connect::Connect;
pub use result::DataType;
pub use result::MysqlResult;
pub use result::QueryResult;
pub use task::Task;
pub use task::TaskEnum;
pub use task::TaskMgmt;
pub use threads::Threads;
pub use threads::Worker;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
