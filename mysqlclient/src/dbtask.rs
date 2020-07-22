pub struct DbTask<RT> {
    /// 数据库名
    pub db_name: String,
    pub sql_str: String,
    pub result: Result<RT, String>,
    pub callback: Box<dyn FnMut(&Result<RT, String>) + Send>,
}

impl<RT> DbTask<RT> {
    pub fn new(
        db_name: String,
        sql_str: String,
        callback: Box<dyn FnMut(&Result<RT, String>) + Send>,
    ) -> Self
    where
        RT: Send + 'static,
    {
        return DbTask {
            db_name,
            sql_str,
            callback,
            result: Err("new".into()),
        };
    }
}

/*
let mut task: DbTask<u64> = DbTask::new(
        "db_name".into(),
        "sql_str".into(),
        Box::new(|result| {
            match result{
                Ok(val)=>{
                    println!("xxxxxxxxxxxx:{}", val);
                }
                Err(err)=>{
                    println!("xxxxxxxxxxxx:{}", err);
                }
            }
        }),
    );
    task.result = Ok(1234567);
    (task.callback)(&task.result);
*/
