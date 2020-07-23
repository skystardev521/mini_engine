use crate::config::ConnConfig;
use crate::config::ThreadConfig;
use crate::task::TaskEnum;
use log::{error, warn};
use std::num::NonZeroU8;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread::Builder;
use std::thread::JoinHandle;

pub struct Threads {
    poll_idx: usize,
    threads: Vec<Worker>,
}

impl Drop for Threads {
    fn drop(&mut self) {
        for worker in &self.threads {
            match worker.sender(TaskEnum::ExitThread) {
                Ok(()) => {}
                Err(_) => {}
            }
        }
        for worker in &self.threads {
            worker.join();
        }
    }
}

impl Threads {
    pub fn new(size: NonZeroU8) -> Self {
        Threads {
            poll_idx: 0,
            threads: Vec::with_capacity(size.get() as usize),
        }
    }

    #[inline]
    fn next_poll_idx(&mut self) {
        if self.poll_idx == self.threads.len() {
            self.poll_idx = 0;
        } else {
            self.poll_idx += 1;
        }
    }

    pub fn len(&self) -> u8 {
        self.threads.len() as u8
    }

    pub fn push(&mut self, worker: Worker) {
        self.threads.push(worker);
    }

    pub fn remove(&mut self, name: &String) -> bool {
        let mut result = false;
        for i in 0..self.threads.len() {
            if self.threads[i].name.eq(name) {
                self.threads.remove(i);
                result = true;
                break;
            }
        }
        result
    }

    pub fn sender(&mut self, msg: TaskEnum) -> bool {
        if self.threads.is_empty() {
            return false;
        }

        let mut result = false;
        let mut temp_msg = msg;

        let init_idx = self.poll_idx;
        loop {
            match self.threads[self.poll_idx].sender(temp_msg) {
                Ok(()) => {
                    result = true;
                    self.next_poll_idx();
                    break;
                }
                Err(res_msg) => {
                    temp_msg = res_msg;
                }
            }
            self.next_poll_idx();
            if self.poll_idx == init_idx {
                break;
            }
        }
        result
    }

    pub fn receiver(&mut self) {
        if self.threads.is_empty() {
            return;
        }
        let mut idx = 0;
        loop {
            if self.threads[idx].receiver() {
                idx += 1;
            } else {
                self.threads.remove(idx);
            }
            if self.threads.len() == idx {
                break;
            }
        }
    }
}

pub struct Worker {
    name: String,
    receiver_max_num: u16,
    join_handle: JoinHandle<()>,
    sender: SyncSender<TaskEnum>,
    receiver: Receiver<TaskEnum>,
}

impl Worker {
    pub fn new(
        name: String,
        thread_config: ThreadConfig,
        conn_config: Vec<ConnConfig>,
        thread_spawn: Box<
            dyn FnOnce(ThreadConfig, Vec<ConnConfig>, Receiver<TaskEnum>, SyncSender<TaskEnum>)
                + Send,
        >,
    ) -> Result<Self, String> {
        let (local_sender, remote_receiver): (SyncSender<TaskEnum>, Receiver<TaskEnum>) =
            mpsc::sync_channel(thread_config.get_channel_size() as usize);

        let (remote_sender, local_receiver): (SyncSender<TaskEnum>, Receiver<TaskEnum>) =
            mpsc::sync_channel(thread_config.get_channel_size() as usize);

        let mut builder = Builder::new().name(name.clone());
        if thread_config.get_stack_size() > 0 {
            builder = builder.stack_size(thread_config.get_stack_size());
        }
        let receiver_max_num = thread_config.get_receiver_max_num();

        match builder.spawn(move || {
            thread_spawn(thread_config, conn_config, remote_receiver, remote_sender);
        }) {
            Ok(join_handle) => Ok(Worker {
                name,
                join_handle,
                sender: local_sender,
                receiver: local_receiver,
                receiver_max_num: receiver_max_num,
            }),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn join(&self) {
        /*
        match self.join_handle.join() {
            Ok(()) => {
                warn!("Worker name:{} Exit", self.name);
            }
            Err(_) => {
                error!("Worker name:{} Exit Error", self.name);
            }
        }
        */
    }

    /// result:false 线程退出
    pub fn receiver(&self) -> bool {
        let mut num: u16 = 0;
        loop {
            match self.receiver.try_recv() {
                Ok(TaskEnum::AlterTask(mut task)) => {
                    (task.callback)(task.result);
                }
                Ok(TaskEnum::QueryTask(mut task)) => {
                    (task.callback)(task.result);
                }
                Err(TryRecvError::Empty) => return true,

                Ok(TaskEnum::ExitThread) => {
                    warn!("Worker name:{} Exit", self.name);
                    return false;
                }
                Err(TryRecvError::Disconnected) => {
                    error!("Worker name:{} Disconnected", self.name);
                    return false;
                }
            }
            num += 1;
            if num == self.receiver_max_num {
                return true;
            }
        }
    }

    #[inline]
    fn sender(&self, msg: TaskEnum) -> Result<(), TaskEnum> {
        match self.sender.try_send(msg) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(res_data)) => {
                warn!("Worker name:{} sender Full", self.name);
                Err(res_data)
            }
            Err(TrySendError::Disconnected(res_data)) => {
                error!("Worker name:{} sender Disconnected", self.name);
                Err(res_data)
            }
        }
    }
}

/*
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
*/

/*


pub trait ThreadsTrait<RT> {
    fn sender(&self, data: DbTask<RT>) -> bool;
}

pub trait WorkerTrait<RT> {
    fn sender(&self, data: DbTask<RT>) -> Result<(), DbTask<RT>>;
}

pub struct Threads {
    pub threads: Vec<Worker>,
}

impl Drop for Threads {
    fn drop(&mut self) {
        for worker in &self.threads {}

        for worker in &self.threads {
            worker.join();
        }
    }
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
    ) -> Result<Self, String> {
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

    pub fn join(&self) {
        match self.joinHandle.join() {
            Ok(_) => {
                wran!("Worker name:{} Exit", self.name);
            }
            Err(_) => {
                error!("Worker name:{} Exit Error", self.name);
            }
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
*/
