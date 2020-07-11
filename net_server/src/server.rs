use crate::config::Config;
use log::error;
use socket::message::NetMsg;
use socket::message::ProtoId;
use std::thread;

pub struct Server<'a> {
    config: &'a Config,
    net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
}

impl<'a> Drop for Server<'a> {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped Server while unwinding");
        } else {
            error!("dropped Server while not unwinding");
        }
    }
}

impl<'a> Server<'a> {
    pub fn new(
        config: &'a Config,
        net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
    ) -> Self {
        Server { config, net_msg_cb }
    }

    pub fn new_net_msg(&mut self, net_msg: NetMsg) {
        //info!("new_net_msg id:{}", net_msg.id);
        match (self.net_msg_cb)(net_msg) {
            Ok(()) => (),
            Err(pid) => error!("net msg cb:{:?}", pid),
        }
    }

    pub fn tick(&mut self) {}
}
