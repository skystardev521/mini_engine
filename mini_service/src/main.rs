use log::error;
use mini_service::Config;
use mini_service::LogicService;
use mini_utils::logger::Logger;
use mini_utils::time;
use std::thread;
use std::thread::Builder;
use std::time::Duration;

const LOG_FILE_DURATION: u64 = 60 * 60 * 1000;

fn main() {
    let mut config = Config::new();
    if let Err(err) = config.read_config(&"confg.txt".into()) {
        println!("config.read_config error:{}", err);
        return;
    }

    let mut log_file_timestamp = time::timestamp();
    match Logger::init(
        &String::from("info"),
        &String::from("logs/mini_service.log"),
    ) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    let logic_builder = Builder::new().name("LogicService".into());
    let _logic_thread = logic_builder.spawn(move || {
        match LogicService::new(config) {
            Ok(logic_service) => {
                logic_service.run();
            }
            Err(err) => error!("LogicService::new Error:{}", err),
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
