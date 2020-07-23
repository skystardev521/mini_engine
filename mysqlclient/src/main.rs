use log::error;
use mysqlclient::ConnConfig;
use mysqlclient::Connect;
use mysqlclient::MysqlResult;
use mysqlclient::QueryResult;
use mysqlclient::Task;
use mysqlclient::TaskEnum;
use mysqlclient::TaskMgmt;
use mysqlclient::ThreadConfig;
use mysqlclient::Threads;
use mysqlclient::Worker;
use std::num::NonZeroU8;
use std::ptr::{self};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use utils::logger::Logger;
use utils::time;

#[macro_use]
mod result;

mod test;

fn main() {
    test();
    /*
    let mut task: Task<u64> = Task::new(
        "db_name".into(),
        "sql_str".into(),
        Box::new(|result| match result {
            Ok(val) => {
                println!("xxxxxxxxxxxx:{}", val);
            }
            Err(err) => {
                println!("xxxxxxxxxxxx:{}", err);
            }
        }),
    );
    task.result = Ok(1234567);
    (task.callback)(&task.result);
    */
    println!("------------------------------------------------");
    //println!("{:?}", tuple_default!(i32, String));

    println!("------------------------------------------------");

    let mut open_log_file_ts = time::timestamp();
    match Logger::init(&String::from("info"), &String::from("logs/mysql.log")) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    /*
    let mut config = Config::new();

    config.set_host(&"127.0.0.1".into());
    config.set_user(&"root".into());
    config.set_password(&"root".into());
    config.set_database(&"dev_db".into());

    let mysql_connect = Connect::new(&config).unwrap();
    if let Err(err) = mysql_connect.connect() {
        println!("connect err:{}", err);
    }

    if let Err(err) = mysql_connect.set_mysql_options() {
        println!("set_mysql_options err:{}", err);
    }

    let sql = format!(
        "insert into test (id,name,text)values({},'name{}','text{}');",
        1, 1, 1
    );
    println!("sql:{}", sql);
    match mysql_connect.alter_data(&sql) {
        Ok(row_num) => {
            println!("mysql_connect.alter_data row num:{}", row_num);
        }
        Err(err) => {
            println!("mysql_connect.alter_data err:{}", err);
        }
    }

    match mysql_connect.query_data(&"SELECT * FROM dev_db.test LIMIT 10;".into()) {
        Ok(result1) => {
            match mysql_connect
                .query_data(&"SELECT * FROM dev_db.test where id = 11111 LIMIT 5;".into())
            {
                Ok(result2) => {
                    let result = query_result!(&result2, 0i32, String::new(), vec![0i8; 0]);
                    if result.is_empty() {
                        println!("result is_empty");
                    } else {
                        println!("result:{:?}", result);
                    }
                }
                Err(err) => {
                    println!("mysql_connect.query_data err:{}", err);
                }
            }
            let result = query_result!(&result1, 0i32, String::new(), vec![0i8; 0]);
            println!("----------------------------------------------------------");
            if result.is_empty() {
                println!("result is_empty");
            } else {
                println!("result:{:?}", result);
            }
        }
        Err(err) => {
            println!("mysql_connect.query_data err:{}", err);
        }
    }
    */
}

pub fn test() {
    let mut thread_config = ThreadConfig::new();
    let mut vec_conn_config: Vec<ConnConfig> = Vec::new();

    thread_config.set_sleep_duration(1000);

    for i in 0..1 {
        let mut config = ConnConfig::new();
        config.set_host(&"127.0.0.1".into());
        config.set_user(&"root".into());
        config.set_password(&"root".into());
        config.set_database(&"dev_db".into());
        vec_conn_config.push(config);
    }

    let mut threads = Threads::new(NonZeroU8::new(thread_config.get_thread_num()).unwrap());
    for i in 0..thread_config.get_thread_num() {
        let name = format!("name{}", i);
        match Worker::new(
            name.clone(),
            thread_config.clone(),
            vec_conn_config.clone(),
            Box::new(
                |thread_config: ThreadConfig,
                 conn_config: Vec<ConnConfig>,
                 receiver: Receiver<TaskEnum>,
                 sender: SyncSender<TaskEnum>| {
                    let mut task_mgmt = TaskMgmt::new(
                        name,
                        thread_config.get_ping_duration(),
                        thread_config.get_sleep_duration(),
                        receiver,
                        sender,
                    );

                    task_mgmt.connect(conn_config);
                    task_mgmt.loop_receiver();
                },
            ),
        ) {
            Ok(worker) => {
                threads.push(worker);
            }
            Err(err) => {
                error!("Worker::new error:{}", err);
            }
        }
    }

    let sql_str = "insert into test (id,name,text)values({},'name{}','text{}');".into();
    let database = format!("{}_{}_{}", "dev_db", "127.0.0.1", 3306);
    let mut task: Task<u64> = Task::new(
        sql_str,
        database,
        Box::new(|result: Result<u64, String>| match result {
            Ok(val) => {
                println!("xxxxxxxxxxxx:{:?}", val);
            }
            Err(err) => {
                println!("xxxxxxxxxxxx:{}", err);
            }
        }),
    );

    let task_enum = TaskEnum::AlterTask(task);
    threads.sender(task_enum);

    let sql_str = "SELECT * FROM dev_db.test LIMIT 10;".into();
    let database = format!("{}_{}_{}", "dev_db", "127.0.0.1", 3306);

    //: Result<QueryResult<MysqlResult>, String>
    let mut task: Task<QueryResult<MysqlResult>> = Task::new(
        sql_str,
        database,
        Box::new(
            |result: Result<QueryResult<MysqlResult>, String>| match result {
                Ok(val) => {
                    /*
                    let result = query_result!(&result2, 0i32, String::new(), vec![0i8; 0]);
                    if result.is_empty() {
                        println!("result is_empty");
                    } else {
                        println!("result:{:?}", result);
                    }
                    */
                }
                Err(err) => {
                    println!("xxxxxxxxxxxx:{}", err);
                }
            },
        ),
    );

    let task_enum = TaskEnum::QueryTask(task);
    threads.sender(task_enum);

    loop {
        threads.receiver();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
