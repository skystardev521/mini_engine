use crate::config::Config;
use crate::lan_service::LanService;
use crate::wan_service::WanService;
use log::error;
use mini_socket::message::MsgEnum;
use std::thread;
use std::time::Duration;

/// 用于把 广域网的数据 转到 局域网服务中
pub struct Service {
    //config: &'a Config,
    wan_service: WanService,
    lan_service: LanService,
    single_max_task_num: u16,
    sleep_duration: Duration,
    //net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
}

impl Drop for Service {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped mini_proxy Service while unwinding");
        } else {
            error!("dropped mini_proxy Service while not unwinding");
        }
    }
}

impl Service {
    pub fn new(
        config: Config,
        //net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
    ) -> Result<Self, String> {
        let wan_service = WanService::new(&config.worker_config, config.wan_listen_config.clone())?;
        let lan_service = LanService::new(&config.worker_config, config.lan_listen_config.clone())?;

        let sleep_duration = config.worker_config.get_sleep_duration();
        let single_max_task_num = config.worker_config.get_single_max_task_num();

        Ok(Service {
            //config,
            wan_service,
            lan_service,
            sleep_duration: sleep_duration,
            single_max_task_num: single_max_task_num,
        })
    }

    pub fn run(&self) {
        loop {
            let mut is_sleep = true;
            if !self.wan_receiver() {
                is_sleep = false;
            }
            if !self.lan_receiver() {
                is_sleep = false;
            }
            if is_sleep {
                thread::sleep(self.sleep_duration);
            }
        }
    }

    /// empty:true data:false
    fn wan_receiver(&self) -> bool {
        let mut num = 0;
        loop {
            match self.wan_service.receiver() {
                None => return true,
                Some(msg) => {
                    self.sender_lan(msg);
                }
            }
            num += 1;
            if num == self.single_max_task_num {
                return false;
            }
        }
    }

    /// empty:true data:false
    fn lan_receiver(&self) -> bool {
        let mut num = 0;
        loop {
            match self.lan_service.receiver() {
                None => return true,
                Some(msg) => {
                    self.sender_wan(msg);
                    num += 1;
                    if num == self.single_max_task_num {
                        return false;
                    }
                }
            }
        }
    }

    fn sender_wan(&self, msg: MsgEnum) {
        self.wan_service.sender(msg);
    }

    fn sender_lan(&self, msg: MsgEnum) {
        self.lan_service.sender(msg);
    }
}