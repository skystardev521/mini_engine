use log::error;

use crate::service::Service;
use config::Config;
use mini_utils::logger::Logger;
use std::thread::{self, Builder};
use std::time::Duration;

mod config;
mod lan_service;
mod lan_tcp_rw;
mod mucid_route;
mod wan_service;
mod wan_tcp_rw;
mod service;

use mini_utils::time;

const LOG_FILE_DURATION: u64 = 60 * 60 * 1000;

fn main() {
    let mut config = Config::new();

    if let Err(err) = config.read_config(&"confg.txt".into()) {
        println!("config.read_config error:{}", err);
        return;
    }

    let mut log_file_timestamp = time::timestamp();
    match Logger::init(&String::from("info"), &String::from("logs/mini_proxy.log")) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    let route_builder = Builder::new().name("route".into());
    let _route_thread = route_builder.spawn(move || {
        match Service::new(config) {
            Ok(service) => {
                let mut mut_service = service;
                mut_service.run();
            }
            Err(err) => error!("Service::new Error:{}", err),
        };
    });

    loop {
        thread::sleep(Duration::from_secs(60));
        if log_file_timestamp + LOG_FILE_DURATION < time::timestamp() {
            log::logger().flush();
            log_file_timestamp = time::timestamp();
        }
    }
}
