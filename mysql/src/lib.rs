mod config;
mod mysql;

pub use config::Config;
pub use mysql::MysqlConnect;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
