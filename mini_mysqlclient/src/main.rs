use mini_utils::logger::Logger;
use mini_utils::time;
use std::ptr::{self};

use crate::qresult::MysqlResult;
use crate::qresult::QueryResult;
use crate::qresult::ITCData;

use crate::config::Config;
use crate::config::ConnConfig;
use crate::service::Service;
use crate::sql_task::SqlTask;
use crate::sql_task::SqlTaskEnum;
use mini_utils::wconfig::WConfig;

#[macro_use]
mod qresult;
mod config;
mod connect;
mod execute;
mod service;
mod sql_task;
mod workers;

fn main() {
    let mut _log_file_timestamp = time::timestamp();
    match Logger::init(&String::from("info"), &String::from("logs/mysql.log")) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }
    test();
}

pub fn test() {
    let mut workers_config = WConfig::new();
    let mut vec_conn_config: Vec<ConnConfig> = Vec::new();

    workers_config.set_sleep_duration(3);

    for _i in 0..10 {
        let mut config = ConnConfig::new();
        config
            .set_host(&"127.0.0.1".into())
            .set_user(&"root".into())
            .set_password(&"root".into())
            .set_database(&"dev_db".into());
        vec_conn_config.push(config);
    }

    let worker_num = 3;
    let config = Config::new(worker_num, workers_config, vec_conn_config);

    let mut service = Service::new(config).unwrap();

    let database = format!("{}_{}_{}", "dev_db", "127.0.0.1", 3306);
    for i in 1..10 {
        let sql_str = format!(
            "insert into test (id,name,text,bin)values({},'name{}','text{}',{:b});",
            i, i, i, i
        );
        let alter_task: SqlTask<u64> = SqlTask::new(
            sql_str,
            database.clone(),
            Box::new(|result: Result<u64, String>| match result {
                Ok(rows) => {
                    println!("affected_rows:{:?}", rows);
                }
                Err(err) => {
                    println!("affected_rows:{}", err);
                }
            }),
        );

        service.sender(SqlTaskEnum::AlterTask(alter_task));
    }

    for _ in 1..5 {
        let sql_str = "SELECT * FROM dev_db.test LIMIT 2;".into();
        let database = format!("{}_{}_{}", "dev_db", "127.0.0.1", 3306);

        let query_task: SqlTask<QueryResult<MysqlResult>> = SqlTask::new(
            sql_str,
            database,
            Box::new(
                |result: Result<QueryResult<MysqlResult>, String>| match result {
                    Ok(query_result) => {
                        let result =
                            query_result!(&query_result, 0i32, String::new(), vec![0i8; 0]);
                        if result.is_empty() {
                            println!("result is_empty");
                        } else {
                            println!("result:{:?}", result);
                        }
                    }
                    Err(err) => {
                        println!("xxxxxxxxxxxx:{}", err);
                    }
                },
            ),
        );

        service.sender(SqlTaskEnum::QueryTask(query_task));
    }

    loop {
        service.receiver();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
