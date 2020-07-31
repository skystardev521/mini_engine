use crate::config::Config;
use crate::conn_service::ConnService;
use log::error;
use mini_socket::message::NetMsg;
use mini_socket::message::ProtoId;
use std::thread;
use std::time::Duration;

pub struct LogicService {
    sleep_duration: Duration,
    single_max_task_num: u16,
    conn_service: ConnService,
    //net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
}

impl Drop for LogicService {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped LogicService while unwinding");
        } else {
            error!("dropped LogicService while not unwinding");
        }
    }
}

impl LogicService {
    pub fn new(
        config: Config,
        //net_msg_cb: &'a mut dyn Fn(NetMsg) -> Result<(), ProtoId>,
    ) -> Result<Self, String> {
        let vec_tcp_connect_config = config.vec_tcp_connect_config.clone();
        let conn_service = ConnService::new(&config.worker_config, vec_tcp_connect_config)?;

        let sleep_duration = config.worker_config.get_sleep_duration();
        let single_max_task_num = config.worker_config.get_single_max_task_num();

        Ok(LogicService {
            conn_service,
            sleep_duration,
            single_max_task_num,
        })
    }

    pub fn run(&self) {
        loop {
            self.tick();
            let mut is_sleep = true;
            if !self.net_receiver() {
                is_sleep = false;
            }

            if is_sleep {
                thread::sleep(self.sleep_duration);
                error!("sleep_duration:{}", self.sleep_duration.as_millis());
            }
        }
    }

    fn net_receiver(&self) -> bool {
        let mut num = 0;
        loop {
            match self.conn_service.receiver() {
                None => return true,
                Some(net_msg) => {
                    error!("receiver:{}", net_msg.sid);
                    self.net_sender(net_msg);
                    num += 1;
                    if num == self.single_max_task_num {
                        return false;
                    }
                }
            }
        }
    }

    fn net_sender(&self, net_msg: NetMsg) -> bool {
        self.conn_service.sender(net_msg)
    }

    /*
    pub fn new_net_msg(&mut self, net_msg: NetMsg) {
        //info!("new_net_msg id:{}", net_msg.id);
        match (self.net_msg_cb)(net_msg) {
            Ok(()) => (),
            Err(pid) => error!("net msg cb:{:?}", pid),
        }
    }
    */

    pub fn tick(&self) {}
}
