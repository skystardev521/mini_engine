/*
use socket::message::NetMsg;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

pub struct Logic<'a, T> {
    config: &'a T,
    receiver: Receiver<NetMsg>,
    sync_sender: SyncSender<NetMsg>,
}

impl<'a, T> Logic<'a, T> {
    pub fn new(config: &'a T, receiver: Receiver<NetMsg>, sync_sender: SyncSender<NetMsg>) -> Self {
        Logic {
            config: config,
            receiver: receiver,
            sync_sender: sync_sender,
        }
    }

    pub fn run(
        &self,
        receiver: Receiver<NetMsg>,
        sync_sender: SyncSender<NetMsg>,
    ) -> Result<(), String> {
        Ok(())
    }
}

*/
