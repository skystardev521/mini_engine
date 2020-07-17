mod anydata;
mod config;
mod mysql;

pub use anydata::test;
pub use config::Config;
pub use mysql::MysqlConnect;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
