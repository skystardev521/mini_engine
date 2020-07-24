use crate::config::ConnConfig;
use crate::config::ThreadConfig;
use crate::task::TaskEnum;
use mini_utils::time;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::thread;

use crate::task_mgmt::RecvRes;
use crate::task_mgmt::TaskMgmt;
use crate::thread_pool::ThreadPool;
use crate::thread_pool::Worker;

use log::error;

pub struct Service {
    thread_pool: ThreadPool,
    thread_config: ThreadConfig,
    vec_conn_config: Vec<ConnConfig>,
}

impl Service {
    pub fn new(thread_config: ThreadConfig, vec_conn_config: Vec<ConnConfig>) -> Self {
        let mut thread_config = thread_config;
        let mut thread_num = thread_config.get_thread_num();
        if thread_num == 0 {
            thread_num = 1;
            thread_config.set_thread_num(1);
        }
        Service {
            thread_config,
            vec_conn_config,
            thread_pool: ThreadPool::new(thread_num),
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        for i in 0..self.thread_config.get_thread_num() {
            let name = format!("mysqlclient_{}", i);
            match Worker::new(
                name.clone(),
                self.thread_config.clone(),
                self.vec_conn_config.clone(),
                thread_closure(name.clone()),
            ) {
                Ok(worker) => {
                    self.thread_pool.push(worker);
                }
                Err(err) => {
                    error!("Worker::new error:{}", err);
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    pub fn recv_task(&mut self) {
        self.thread_pool.receiver();
    }

    pub fn sender(&mut self, task_enum: TaskEnum) -> bool {
        self.thread_pool.sender(task_enum)
    }
}

fn thread_closure(
    name: String,
) -> Box<dyn FnOnce(ThreadConfig, Vec<ConnConfig>, Receiver<TaskEnum>, SyncSender<TaskEnum>) + Send>
{
    Box::new(
        |thread_config: ThreadConfig,
         conn_config: Vec<ConnConfig>,
         receiver: Receiver<TaskEnum>,
         sender: SyncSender<TaskEnum>| {
            let mut task_mgmt =
                TaskMgmt::new(name, thread_config.get_sleep_duration(), receiver, sender);

            task_mgmt.connect(conn_config);

            let mut last_ping_ts = time::timestamp();
            let ping_duration = thread_config.get_ping_duration().as_secs();

            loop {
                match task_mgmt.receiver() {
                    RecvRes::Empty => {
                        if last_ping_ts + ping_duration < time::timestamp() {
                            task_mgmt.ping_connect();
                            last_ping_ts = time::timestamp();
                        }
                        thread::sleep(thread_config.get_sleep_duration());
                    }
                    RecvRes::TaskData => {
                        continue;
                    }
                    RecvRes::ExitThread => break,
                }
            }
        },
    )
}

/*
#[macro_use]
//mod result;

use crate::result::QueryResult;
use crate::result::MysqlResult;
use crate::result::CellValue;
use crate::task::Task;

use crate::task::TaskEnum;
use crate::service::Service;
use crate::config::ConnConfig;
use crate::config::ThreadConfig;


pub fn test() {
    let mut thread_config = ThreadConfig::new();
    let mut vec_conn_config: Vec<ConnConfig> = Vec::new();

    thread_config.set_sleep_duration(1000).set_thread_num(5);

    for i in 0..10 {
        let mut config = ConnConfig::new();
        config.set_host(&"127.0.0.1".into());
        config.set_user(&"root".into());
        config.set_password(&"root".into());
        config.set_database(&"dev_db".into());
        vec_conn_config.push(config);
    }

    let mut service = Service::new(thread_config, vec_conn_config);
    service.init();

    let database = format!("{}_{}_{}", "dev_db", "127.0.0.1", 3306);
    for i in 1..10 {
        let sql_str = format!(
            "insert into test (id,name,text)values({},'name{}','text{}');",
            i, i, i
        );
        let alter_task: Task<u64> = Task::new(
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

        service.sender(TaskEnum::AlterTask(alter_task));
    }

    for _ in 1..500 {
        let sql_str = "SELECT * FROM dev_db.test LIMIT 10;".into();
        let database = format!("{}_{}_{}", "dev_db", "127.0.0.1", 3306);

        let query_task: Task<QueryResult<MysqlResult>> = Task::new(
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

        service.sender(TaskEnum::QueryTask(query_task));
    }

    loop {
        service.recv_task();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
*/
