use crate::config::Config;

use mini_utils::time;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::thread;

use crate::execute::Execute;
use crate::execute::RecvRes;
use crate::task::TaskEnum;
use crate::workers::Workers;
use mini_utils::worker::Worker;

use log::error;

pub struct Service {
    workers: Workers,
}

/// 发送Task接收Task结果
impl Service {
    pub fn new(config: Config) -> Result<Self, String> {
        let worker_num = config.workers_config.get_worker_num();
        let single_max_task_num = config.workers_config.get_single_max_task_num();
        let mut service = Service {
            workers: Workers::new(worker_num, single_max_task_num),
        };
        service.init(config)?;
        Ok(service)
    }

    fn init(&mut self, config: Config) -> Result<(), String> {
        let worker_num = config.workers_config.get_worker_num();
        for i in 0..worker_num {
            let name = format!("mysqlclient_{}", i);
            match Worker::new(
                name.clone(),
                config.workers_config.get_stack_size(),
                config.workers_config.get_channel_size(),
                worker_closure(name.clone(), config.clone()),
            ) {
                Ok(worker) => {
                    self.workers.push(worker);
                }
                Err(err) => {
                    error!("Worker::new error:{}", err);
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    pub fn receiver(&mut self) {
        self.workers.receiver();
    }

    pub fn sender(&mut self, task_enum: TaskEnum) -> bool {
        self.workers.sender(task_enum)
    }
}

fn worker_closure(
    name: String,
    config: Config,
) -> Box<dyn FnOnce(Receiver<TaskEnum>, SyncSender<TaskEnum>) + Send> {
    Box::new(
        move |receiver: Receiver<TaskEnum>, sender: SyncSender<TaskEnum>| {
            let mut execute = Execute::new(
                name,
                config.workers_config.get_sleep_duration(),
                receiver,
                sender,
            );

            execute.connect(config.vec_connect_config);

            let mut last_ping_timestamp = time::timestamp();
            let ping_interval = config.workers_config.get_ping_interval().as_secs();

            loop {
                match execute.receiver() {
                    RecvRes::Empty => {
                        if last_ping_timestamp + ping_interval < time::timestamp() {
                            execute.ping_connect();
                            last_ping_timestamp = time::timestamp();
                        }
                        thread::sleep(config.workers_config.get_sleep_duration());
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
    let mut workers_config = ThreadConfig::new();
    let mut vec_conn_config: Vec<ConnConfig> = Vec::new();

    workers_config.set_sleep_duration(1000).set_worker_num(5);

    for i in 0..10 {
        let mut config = ConnConfig::new();
        config.set_host(&"127.0.0.1".into());
        config.set_user(&"root".into());
        config.set_password(&"root".into());
        config.set_database(&"dev_db".into());
        vec_conn_config.push(config);
    }

    let mut service = Service::new(workers_config, vec_conn_config);
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
