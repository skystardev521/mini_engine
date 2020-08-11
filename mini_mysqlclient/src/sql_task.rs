use crate::qresult::MysqlResult;
use crate::qresult::QueryResult;
pub enum SqlTaskEnum {
    AlterTask(SqlTask<u64>),
    QueryTask(SqlTask<QueryResult<MysqlResult>>),
}

pub struct SqlTask<RT> {
    /// 数据库Id
    pub sql_str: String,
    /// db_host_port
    pub database: String,
    pub result: Result<RT, String>,
    pub callback: Box<dyn FnMut(Result<RT, String>) + Send>,
}

impl<RT> SqlTask<RT> {
    pub fn new(
        sql_str: String,
        database: String,
        callback: Box<dyn FnMut(Result<RT, String>) + Send>,
    ) -> Self
    where
        RT: Send + 'static,
    {
        return SqlTask {
            sql_str,
            database,
            callback,
            result: Err("new".into()),
        };
    }
}
