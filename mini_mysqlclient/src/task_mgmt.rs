use crate::config::ConnConfig;
use crate::connect::Connect;
use crate::task::TaskEnum;
use log::{error, warn};
use std::collections::HashMap;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::time::Duration;

pub enum RecvRes {
    Empty,
    TaskData,
    ExitThread,
}

pub struct TaskMgmt {
    name: String,
    sleep_duration: Duration,
    receiver: Receiver<TaskEnum>,
    sender: SyncSender<TaskEnum>,
    conn_hm: HashMap<String, Connect>,
}

impl TaskMgmt {
    pub fn new(
        name: String,
        sleep_duration: Duration,
        receiver: Receiver<TaskEnum>,
        sender: SyncSender<TaskEnum>,
    ) -> Self {
        TaskMgmt {
            name,
            sender,
            receiver,
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
                Ok(conn) => match conn.connect() {
                    Ok(()) => {
                        println!("connect database:{} Succ", database);
                        self.conn_hm.insert(database, conn);
                    }
                    Err(err) => {
                        error!("connect database:{} err:{}", database, err);
                    }
                },
                Err(err) => error!("connect database:{} err:{}", database, err),
            }
        }
    }

    pub fn receiver(&self) -> RecvRes {
        match self.receiver.try_recv() {
            Err(TryRecvError::Empty) => {
                return RecvRes::Empty;
            }
            Ok(TaskEnum::QueryTask(mut task)) => {
                if let Some(conn) = self.conn_hm.get(&task.database) {
                    task.result = conn.query_data(&task.sql_str);
                    self.sender(TaskEnum::QueryTask(task));
                } else {
                    task.result = Err("database not exist".into());
                    self.sender(TaskEnum::QueryTask(task));
                }
                return RecvRes::TaskData;
            }
            Ok(TaskEnum::AlterTask(mut task)) => {
                if let Some(conn) = self.conn_hm.get(&task.database) {
                    task.result = conn.alter_data(&task.sql_str);
                    self.sender(TaskEnum::AlterTask(task));
                } else {
                    task.result = Err("database not exist".into());
                    self.sender(TaskEnum::AlterTask(task));
                }
                return RecvRes::TaskData;
            }
            Ok(TaskEnum::ExitThread) => {
                warn!("Worker name:{} receiver TaskEnum::ExitThread", self.name);
                return RecvRes::ExitThread;
            }
            Err(TryRecvError::Disconnected) => {
                warn!("Worker name:{} receiver Disconnected", self.name);
                return RecvRes::ExitThread;
            }
        }
    }

    pub fn ping_connect(&self) {
        for conn in self.conn_hm.values() {
            conn.ping();
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
