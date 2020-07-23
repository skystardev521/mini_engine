use crate::result::MysqlResult;
use crate::result::QueryResult;
use crate::ConnConfig;
use crate::Connect;
use log::{error, warn};
use std::collections::HashMap;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::time::Duration;

pub enum TaskEnum {
    ExitThread,
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

pub struct TaskMgmt {
    name: String,
    ping_duration: Duration,
    sleep_duration: Duration,
    receiver: Receiver<TaskEnum>,
    sender: SyncSender<TaskEnum>,
    conn_hm: HashMap<String, Connect>,
}

impl TaskMgmt {
    pub fn new(
        name: String,
        ping_duration: Duration,
        sleep_duration: Duration,
        receiver: Receiver<TaskEnum>,
        sender: SyncSender<TaskEnum>,
    ) -> Self {
        TaskMgmt {
            name,
            sender,
            receiver,
            ping_duration,
            sleep_duration,
            conn_hm: HashMap::new(),
        }
    }

    pub fn connect(&mut self, vec_config: Vec<ConnConfig>) {
        for cfg in vec_config {
            let db = match cfg.get_database() {
                Some(val) => val.to_string_lossy().to_string(),
                None => {
                    error!("database config database is null");
                    continue;
                }
            };
            let host = match cfg.get_host() {
                Some(val) => val.to_string_lossy().to_string(),
                None => {
                    error!("database config host is null");
                    continue;
                }
            };
            let database = format!("{}_{}_{}", db, host, cfg.get_port());

            match Connect::new(cfg) {
                Ok(conn) => {
                    self.conn_hm.insert(database, conn);
                }
                Err(err) => error!("connect database:{} err:{}", database, err),
            }
        }
    }

    pub fn loop_receiver(&self) {
        loop {
            match self.receiver.try_recv() {
                Ok(TaskEnum::QueryTask(mut task)) => {
                    if let Some(conn) = self.conn_hm.get(&task.database) {
                        task.result = conn.query_data(&task.sql_str);
                        self.sender(TaskEnum::QueryTask(task));
                    } else {
                        task.result = Err("db_id not exist".into());
                        self.sender(TaskEnum::QueryTask(task));
                    }
                }
                Ok(TaskEnum::AlterTask(mut task)) => {
                    if let Some(conn) = self.conn_hm.get(&task.database) {
                        task.result = conn.alter_data(&task.sql_str);
                        self.sender(TaskEnum::AlterTask(task));
                    } else {
                        task.result = Err("db_id not exist".into());
                        self.sender(TaskEnum::AlterTask(task));
                    }
                }
                Err(TryRecvError::Empty) => std::thread::sleep(self.sleep_duration),
                Ok(TaskEnum::ExitThread) => {
                    warn!("Worker name:{} receiver TaskEnum::ExitThread", self.name);
                    break;
                }

                Err(TryRecvError::Disconnected) => {
                    warn!("Worker name:{} receiver Disconnected", self.name);
                    return;
                }
            }
        }
    }

    fn sender(&self, task_enum: TaskEnum) {
        let mut new_msg = task_enum;
        loop {
            match self.sender.try_send(new_msg) {
                Ok(()) => break,
                Err(TrySendError::Full(res_msg)) => {
                    new_msg = res_msg;
                    std::thread::sleep(self.sleep_duration);
                    warn!("Worker name:{} sender Full", self.name);
                }
                Err(TrySendError::Disconnected(_)) => {
                    error!("Worker name:{} sender Disconnected", self.name);
                    return;
                }
            }
        }
    }
}
