use crate::config::Config;
use log::error;
use mini_socket::message::NetMsg;
use mini_socket::message::ProtoId;
use std::thread;

/// 用于把 广域网的数据 转到 局域网服务
pub struct RouteService<'a> {
    config: &'a Config,
    net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
}

impl<'a> Drop for RouteService<'a> {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped mini_proxy RouteService while unwinding");
        } else {
            error!("dropped mini_proxy RouteService while not unwinding");
        }
    }
}

impl<'a> RouteService<'a> {
    pub fn new(
        config: &'a Config,
        net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
    ) -> Self {
        RouteService { config, net_msg_cb }
    }

    pub fn new_net_msg(&mut self, net_msg: NetMsg) {
        //info!("new_net_msg id:{}", net_msg.id);
        (self.net_msg_cb)(net_msg);
    }

    pub fn tick(&mut self) {}
}
