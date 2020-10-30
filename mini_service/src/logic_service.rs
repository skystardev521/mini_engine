use crate::config::Config;
use crate::conn_service::ConnService;
use log::{error, warn};
use mini_socket::exc_kind::SProtoId;
use std::thread;
use std::time::Duration;
use crate::proto_head::{MsgEnum, NetMsg};

pub struct LogicService {
    sleep_duration: Duration,
    single_max_task_num: u16,
    conn_service: ConnService,
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
    pub fn new(config: Config) -> Result<Self, String> {
        let vec_tcp_connect_config = config.vec_tcp_connect_config.clone();
        let conn_service = ConnService::new(&config.wconfig, vec_tcp_connect_config)?;

        let sleep_duration = config.wconfig.get_sleep_duration();
        let single_max_task_num = config.wconfig.get_single_max_task_num();

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
            }
        }
    }

    fn net_receiver(&self) -> bool {
        let mut num = 0;
        loop {
            match self.conn_service.receiver() {
                None => return true,
                Some(msg) => {
                    self.net_sender(msg);
                }
            }
            num += 1;
            if num == self.single_max_task_num {
                return false;
            }
        }
    }

    fn net_sender(&self, msg: MsgEnum) -> bool {
        self.conn_service.sender(msg)
    }

    pub fn tick(&self) {}
}
