use crate::result::MysqlResult;
use crate::result::QueryResult;
pub enum TaskEnum {
    AlterTask(Task<u64>),
    QueryTask(Task<QueryResult<MysqlResult>>),
}

pub struct Task<RT> {
    /// 数据库Id
    pub sql_str: String,
    /// database_host_port
    pub database: String,
    pub result: Result<RT, String>,
    pub callback: Box<dyn FnMut(Result<RT, String>) + Send>,
}

impl<RT> Task<RT> {
    pub fn new(
        sql_str: String,
        database: String,
        callback: Box<dyn FnMut(Result<RT, String>) + Send>,
    ) -> Self
    where
        RT: Send + 'static,
    {
        return Task {
            sql_str,
            database,
            callback,
            result: Err("new".into()),
        };
    }
}
