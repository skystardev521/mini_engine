use crate::config::Config;
use log::error;
use mini_socket::message::NetMsg;
use mini_socket::message::ProtoId;
use std::thread;

pub struct Service<'a> {
    config: &'a Config,
    net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
}

impl<'a> Drop for Service<'a> {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped mini_proxy service while unwinding");
        } else {
            error!("dropped mini_proxy service while not unwinding");
        }
    }
}

impl<'a> Service<'a> {
    pub fn new(
        config: &'a Config,
        net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
    ) -> Self {
        Service { config, net_msg_cb }
    }

    pub fn new_net_msg(&mut self, net_msg: NetMsg) {
        //info!("new_net_msg id:{}", net_msg.id);
        (self.net_msg_cb)(net_msg);
    }

    pub fn tick(&mut self) {}
}
