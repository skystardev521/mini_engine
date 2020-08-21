use crate::config::ConnConfig;
use crate::connect::Connect;
use crate::sql_task::SqlTaskEnum;
use log::error;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::time::Duration;

pub(crate) enum RecvRes {
    Empty,
    TaskData,
    ExitThread,
}

/// 读取Task执行sql语句反回任务结果
pub(crate) struct Execute {
    name: String,
    sleep_duration: Duration,
    receiver: Receiver<SqlTaskEnum>,
    sender: SyncSender<SqlTaskEnum>,
    conn_hm: HashMap<String, Connect>,
}

impl Execute {
    pub fn new(
        name: String,
        sleep_duration: Duration,
        receiver: Receiver<SqlTaskEnum>,
        sender: SyncSender<SqlTaskEnum>,
    ) -> Self {
        Execute {
            name,
            sender,
            receiver,
            sleep_duration,
            conn_hm: HashMap::new(),
        }
    }

    pub fn connect(&mut self, vec_config: Vec<ConnConfig>) {
        for config in vec_config {
            let db = match config.get_database() {
                Some(val) => val.to_string_lossy().to_string(),
                None => {
                    error!("database config database is null");
                    continue;
                }
            };
            let host = match config.get_host() {
                Some(val) => val.to_string_lossy().to_string(),
                None => {
                    error!("database config host is null");
                    continue;
                }
            };
            let database = format!("{}_{}_{}", db, host, config.get_port());

            match Connect::new(config) {
                Ok(conn) => match conn.connect() {
                    Ok(()) => {
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
            Ok(SqlTaskEnum::QueryTask(mut sql_task)) => {
                if let Some(conn) = self.conn_hm.get(&sql_task.database) {
                    sql_task.result = conn.query_data(&sql_task.sql_str);
                    self.sender(SqlTaskEnum::QueryTask(sql_task));
                } else {
                    sql_task.result = Err("db not exist".into());
                    self.sender(SqlTaskEnum::QueryTask(sql_task));
                }
                return RecvRes::TaskData;
            }
            Ok(SqlTaskEnum::AlterTask(mut sql_task)) => {
                if let Some(conn) = self.conn_hm.get(&sql_task.database) {
                    sql_task.result = conn.alter_data(&sql_task.sql_str);
                    self.sender(SqlTaskEnum::AlterTask(sql_task));
                } else {
                    sql_task.result = Err("db not exist".into());
                    self.sender(SqlTaskEnum::AlterTask(sql_task));
                }
                return RecvRes::TaskData;
            }
            Err(TryRecvError::Disconnected) => {
                error!("Worker Name:{} Receiver Disconnected", self.name);
                return RecvRes::ExitThread;
            }
        }
    }

    pub fn ping_connect(&self) {
        for conn in self.conn_hm.values() {
            conn.ping();
        }
    }

    fn sender(&self, task_enum: SqlTaskEnum) {
        let mut new_msg = task_enum;
        loop {
            match self.sender.try_send(new_msg) {
                Ok(()) => break,
                Err(TrySendError::Full(res_msg)) => {
                    new_msg = res_msg;
                    std::thread::sleep(self.sleep_duration);
                    error!("Worker Name:{} Sender Full", self.name);
                }
                Err(TrySendError::Disconnected(_)) => {
                    error!("Worker Name:{} Sender Disconnected", self.name);
                    return;
                }
            }
        }
    }
}
