/*
use log::{error, warn};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread::Builder;
use std::thread::JoinHandle;

pub struct Worker<MT, FT> {
    name: String,
    sender: SyncSender<MT>,
    receiver: Receiver<MT>,
    _join_handle: JoinHandle<FT>,
}

impl<MT, FT> Worker<MT, FT> {
    pub fn new(
        name: String,
        stack_size: usize,
        channel_size: u16,
        worker_run: Box<dyn FnOnce(Receiver<MT>, SyncSender<MT>) -> FT + Send>,
    ) -> Result<Self, String>
    where
        MT: Send + 'static,
        FT: Send + 'static,
    {
        let (local_sender, remote_receiver): (SyncSender<MT>, Receiver<MT>) =
            mpsc::sync_channel(channel_size as usize);

        let (remote_sender, local_receiver): (SyncSender<MT>, Receiver<MT>) =
            mpsc::sync_channel(channel_size as usize);

        let mut builder = Builder::new().name(name.clone());
        if stack_size > 0 {
            builder = builder.stack_size(stack_size);
        }

        match builder.spawn(move || worker_run(remote_receiver, remote_sender)) {
            Ok(_join_handle) => Ok(Worker {
                name,
                _join_handle,
                sender: local_sender,
                receiver: local_receiver,
            }),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn receiver(&self) -> Option<MT> {
        loop {
            match self.receiver.try_recv() {
                Ok(msg) => return Some(msg),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    error!("Worker name:{} Disconnected", self.name);
                    return None;
                }
            }
        }
    }

    #[inline]
    pub fn sender(&self, msg: MT) -> bool {
        match self.sender.try_send(msg) {
            Ok(()) => return true,
            Err(TrySendError::Full(_)) => {
                warn!("Worker name:{} sender Full", self.name);
                return false;
            }
            Err(TrySendError::Disconnected(_)) => {
                error!("Worker name:{} sender Disconnected", self.name);
                return false;
            }
        }
    }
}

/*
use crate::config::WorkerConfig;
use log::{error, warn};
use mini_socket::message::NetMsg;
use mini_socket::tcp_listen_config::TcpListenConfig;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread::Builder;
use std::thread::JoinHandle;

pub struct Worker {
    name: String,
    single_max_task_num: u16,
    _join_handle: JoinHandle<()>,
    sender: SyncSender<NetMsg>,
    receiver: Receiver<NetMsg>,
}

impl Worker {
    pub fn new(
        name: String,
        worker_config: WorkerConfig,
        tcp_listen_config: TcpListenConfig,
        worker_run: Box<
            dyn FnOnce(WorkerConfig, TcpListenConfig, Receiver<NetMsg>, SyncSender<NetMsg>) + Send,
        >,
    ) -> Result<Self, String> {
        let (local_sender, remote_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
            mpsc::sync_channel(worker_config.get_channel_size() as usize);

        let (remote_sender, local_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
            mpsc::sync_channel(worker_config.get_channel_size() as usize);

        let mut builder = Builder::new().name(name.clone());
        if worker_config.get_stack_size() > 0 {
            builder = builder.stack_size(worker_config.get_stack_size());
        }
        let single_max_task_num = worker_config.get_single_max_task_num();

        match builder.spawn(move || {
            worker_run(
                worker_config,
                tcp_listen_config,
                remote_receiver,
                remote_sender,
            );
        }) {
            Ok(_join_handle) => Ok(Worker {
                name,
                _join_handle,
                sender: local_sender,
                receiver: local_receiver,
                single_max_task_num: single_max_task_num,
            }),
            Err(err) => Err(err.to_string()),
        }
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
                Err(TryRecvError::Disconnected) => {
                    error!("Worker name:{} Disconnected", self.name);
                    return false;
                }
            }
            num += 1;
            if num == self.single_max_task_num {
                return true;
            }
        }
    }

    #[inline]
    pub fn sender(&self, msg: NetMsg) -> bool {
        match self.sender.try_send(msg) {
            Ok(()) => return true,
            Err(TrySendError::Full(res_data)) => {
                warn!("Worker name:{} sender Full", self.name);
                return false;
            }
            Err(TrySendError::Disconnected(res_data)) => {
                error!("Worker name:{} sender Disconnected", self.name);
                return false;
            }
        }
    }
}
*/
