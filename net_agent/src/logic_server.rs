use socket::message::NetMsg;

use crate::logic_config::LogicConfig;
use log::info;
use std::thread;

pub struct LogicServer<'a> {
    config: &'a LogicConfig,
    new_net_msg_cb: &'a mut dyn Fn(NetMsg),
}

impl<'a> Drop for LogicServer<'a> {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("dropped LogicServer while unwinding");
        } else {
            println!("dropped LogicServer while not unwinding");
        }
    }
}

impl<'a> LogicServer<'a> {
    pub fn new(config: &'a LogicConfig, new_net_msg_cb: &'a mut dyn Fn(NetMsg)) -> Self {
        LogicServer {
            config,
            new_net_msg_cb,
        }
    }

    pub fn new_net_msg(&mut self, net_msg: NetMsg) {
        info!("net_msg id:{}", net_msg.id);
        (self.new_net_msg_cb)(net_msg);
    }
}
