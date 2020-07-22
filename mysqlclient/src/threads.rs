use crate::dbtask::DbTask;
use crate::result::QueryResult;
use log::{error, warn};
use std::num::NonZeroU16;
use std::num::NonZeroU8;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread::Builder;
use std::thread::JoinHandle;
use std::thread::Thread;

#[macro_use]
use crate::result;

pub trait ThreadsTrait<RT> {
    fn sender(&self, data: DbTask<RT>) -> bool;
}

pub trait WorkerTrait<RT> {
    fn sender(&self, data: DbTask<RT>) -> Result<(), DbTask<RT>>;
}

pub struct Threads {
    pub threads: Vec<Worker>,
}

impl Threads {
    pub fn new(size: NonZeroU8) -> Self {
        Threads {
            threads: Vec::with_capacity(size.get() as usize),
        }
    }

    pub fn push(&mut self, worker: Worker) {
        self.threads.push(worker);
    }

    /*
    pub fn receiver() -> Result<(), String> {

        let mut result = false;
        let mut temp_data = data;
        for worker in &self.threads {
            match worker.sender(temp_data) {
                Ok(()) => {
                    result = true;
                    break;
                }
                Err(TrySendError::Full(res_data)) => {
                    temp_data = res_data;
                    warn!("thread channel:{} try_send Full", worker.name);
                }
                Err(TrySendError::Disconnected(res_data)) => {
                    temp_data = res_data;
                    error!("thread channel:{}  try_send Disconnected", worker.name);
                }
            }
        }
        result

        //task.result = Ok(1234567);
        //(task.callback)(&task.result);
    }
    */
}

impl ThreadsTrait<u64> for Threads {
    fn sender(&self, data: DbTask<u64>) -> bool {
        let mut result = false;
        let mut temp_data = data;
        for worker in &self.threads {
            match worker.sender(temp_data) {
                Ok(()) => {
                    result = true;
                    break;
                }
                Err(res_data) => {
                    temp_data = res_data;
                }
            }
        }
        result
    }
}

impl ThreadsTrait<QueryResult> for Threads {
    fn sender(&self, data: DbTask<QueryResult>) -> bool {
        let mut result = false;
        let mut temp_data = data;
        for worker in &self.threads {
            match worker.sender(temp_data) {
                Ok(()) => {
                    result = true;
                    break;
                }
                Err(res_data) => {
                    temp_data = res_data;
                }
            }
        }
        result
    }
}

pub struct Worker {
    name: String,
    receiver_max_num: u16,
    joinHandle: JoinHandle<()>,
    alter_db_sender: SyncSender<DbTask<u64>>,
    alter_db_receiver: Receiver<DbTask<u64>>,
    query_db_sender: SyncSender<DbTask<QueryResult>>,
    query_db_receiver: Receiver<DbTask<QueryResult>>,
}

fn spawn(
    alter_db_sender: SyncSender<DbTask<u64>>,
    alter_db_receiver: Receiver<DbTask<u64>>,
    query_db_sender: SyncSender<DbTask<QueryResult>>,
    query_db_receiver: Receiver<DbTask<QueryResult>>,
) {
}

impl Worker {
    fn new(
        name: String,
        builder: Builder,
        channel_size: NonZeroU16,
        receiver_max_num: NonZeroU16,
        alter_db_sender: SyncSender<DbTask<u64>>,
        alter_db_receiver: Receiver<DbTask<u64>>,
        query_db_sender: SyncSender<DbTask<QueryResult>>,
        query_db_receiver: Receiver<DbTask<QueryResult>>,
        thread_spawn: Box<
            dyn FnOnce(
                    SyncSender<DbTask<u64>>,
                    Receiver<DbTask<u64>>,
                    SyncSender<DbTask<QueryResult>>,
                    Receiver<DbTask<QueryResult>>,
                ) + Send,
        >,
    ) -> Result<Self, String>
/*
    where
        F: FnOnce() -> (),
        F: Send + 'static,
        //RT: Send + 'static,
        */ {
        let (local_alter_db_sender, remote_alter_db_receiver): (
            SyncSender<DbTask<u64>>,
            Receiver<DbTask<u64>>,
        ) = mpsc::sync_channel(channel_size.get() as usize);

        let (remote_alter_db_sender, local_alter_db_receiver): (
            SyncSender<DbTask<u64>>,
            Receiver<DbTask<u64>>,
        ) = mpsc::sync_channel(channel_size.get() as usize);

        let (local_query_db_sender, remote_query_db_receiver): (
            SyncSender<DbTask<QueryResult>>,
            Receiver<DbTask<QueryResult>>,
        ) = mpsc::sync_channel(channel_size.get() as usize);

        let (remote_query_db_sender, local_query_db_receiver): (
            SyncSender<DbTask<QueryResult>>,
            Receiver<DbTask<QueryResult>>,
        ) = mpsc::sync_channel(channel_size.get() as usize);

        match builder.spawn(move || {
            /*
            thread_spawn(
                remote_alter_db_sender,
                remote_alter_db_receiver,
                remote_query_db_sender,
                remote_query_db_receiver,
            );
            */
        }) {
            Ok(joinHandle) => Ok(Worker {
                name,
                joinHandle,
                receiver_max_num: receiver_max_num.get(),
                alter_db_sender: local_alter_db_sender,
                alter_db_receiver: local_alter_db_receiver,
                query_db_sender: local_query_db_sender,
                query_db_receiver: local_query_db_receiver,
            }),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn receiver(&self) {
        self.loop_receiver("alter_db_receiver", &self.alter_db_receiver);
        self.loop_receiver("query_db_receiver", &self.query_db_receiver);
    }

    fn loop_receiver<T>(&self, receiver_name: &str, receiver: &Receiver<DbTask<T>>) {
        let mut num: u16 = 0;
        loop {
            match receiver.try_recv() {
                Ok(mut dbtask) => {
                    (dbtask.callback)(&dbtask.result);
                }
                Err(TryRecvError::Empty) => break,

                Err(TryRecvError::Disconnected) => {
                    error!(
                        "Worker name:{} {}.try_recv Disconnected",
                        receiver_name, self.name
                    );
                    break;
                }
            }
            num += 1;
            if num == self.receiver_max_num {
                break;
            }
        }
    }

    #[inline]
    fn sender_task<T>(
        &self,
        data: DbTask<T>,
        sender_name: &str,
        sender: &SyncSender<DbTask<T>>,
    ) -> Result<(), DbTask<T>> {
        match sender.try_send(data) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(res_data)) => {
                warn!("Worker name :{} {} Full", sender_name, self.name);
                Err(res_data)
            }
            Err(TrySendError::Disconnected(res_data)) => {
                error!("Worker name :{} {} Disconnected", sender_name, self.name);
                Err(res_data)
            }
        }
    }
}

impl WorkerTrait<u64> for Worker {
    #[inline]
    fn sender(&self, data: DbTask<u64>) -> Result<(), DbTask<u64>> {
        self.sender_task::<u64>(data, "alter_db_sender", &self.alter_db_sender)
    }
}

impl WorkerTrait<QueryResult> for Worker {
    #[inline]
    fn sender(&self, data: DbTask<QueryResult>) -> Result<(), DbTask<QueryResult>> {
        self.sender_task::<QueryResult>(data, "query_db_sender", &self.query_db_sender)
    }
}
