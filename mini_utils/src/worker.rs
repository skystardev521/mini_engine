use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread::Builder;
use std::thread::JoinHandle;

pub enum RecvResEnum<MT> {
    Empty,
    Data(MT),
    Disconnected,
}

pub enum SendResEnum<MT> {
    Success,
    Full(MT),
    Disconnected(MT),
}

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
        channel_size: u32,
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

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn receiver(&self) -> RecvResEnum<MT> {
        match self.receiver.try_recv() {
            Ok(msg) => return RecvResEnum::Data(msg),
            Err(TryRecvError::Empty) => return RecvResEnum::Empty,
            Err(TryRecvError::Disconnected) => return RecvResEnum::Disconnected,
        }
    }

    #[inline]
    pub fn sender(&self, msg: MT) -> SendResEnum<MT> {
        match self.sender.try_send(msg) {
            Ok(()) => return SendResEnum::Success,
            Err(TrySendError::Full(msg)) => {
                return SendResEnum::Full(msg);
            }
            Err(TrySendError::Disconnected(msg)) => {
                return SendResEnum::Disconnected(msg);
            }
        }
    }
}
