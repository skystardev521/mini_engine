use socket::message::NetMsg;

use crate::config::Config;
use log::error;
use std::thread;

pub struct Server<'a> {
    config: &'a Config,
    net_msg_cb: &'a mut dyn Fn(NetMsg),
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
    pub fn new(config: &'a Config, net_msg_cb: &'a mut dyn Fn(NetMsg)) -> Self {
        Server { config, net_msg_cb }
    }

    pub fn new_net_msg(&mut self, net_msg: NetMsg) {
        //info!("new_net_msg id:{}", net_msg.id);
        (self.net_msg_cb)(net_msg);
    }

    pub fn tick(&mut self) {}
}
