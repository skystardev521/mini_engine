//use std::thread;
use utils::logger;
//use utils::time;
pub mod test_tcp;

use std::io::prelude::*;
use std::net::TcpStream;
use utils::time;

pub struct ThreadPool {
    //handlers: Vec<thread::JoinHandle<()>>,
}

fn main() {
    match logger::Logger::init(&String::from("info"), &String::from("logs/test_log.log"), 1) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    let ts1 = time::timestamp();
    println!("start connect time:{}", ts1);
    match TcpStream::connect("www.baidu.com:80") {
        Ok(socket) => {
            let ts2 = time::timestamp();
            println!("end connect time:{}", ts2);
            println!("ts2 - ts1 time:{}", ts2 - ts1);
            //socket.write(b"GET / HTTP/1.0\n\n").unwrap(); //获取发送网页

            /*
            let mut buffer = vec![];
            let response = socket.read_to_end(&mut buffer).unwrap(); //得到结果
            let s = String::from_utf8(buffer).unwrap(); //转换结果为字符串.
            println!("{}", s);
            */
        }
        Err(err) => {
            println!("connect error:{}", err);
        }
    } //

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

    let mut hm: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
}
