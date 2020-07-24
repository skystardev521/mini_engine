//use std::thread;
use mini_utils::logger::Logger;
//use mini_utils::time;
pub mod test_tcp;

pub struct ThreadPool {
    //handlers: Vec<thread::JoinHandle<()>>,
}

fn main() {
    match Logger::init(&String::from("info"), &String::from("logs/test_log.log")) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    test_tcp::test();

    /*
        let mut thread_pool: Vec<thread::JoinHandle<()>> = vec![];

        for _ in 0..16 {
            thread_pool.push(thread::spawn(move || {
                for i in 0..99999999 {
                    info!("thread_id-->{}:{:?}", i, std::thread::current().id());
                    //std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }));
        }
    */

    //let mut hm: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
}
