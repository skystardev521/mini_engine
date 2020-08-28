//use std::thread;
use mini_utils::logger::Logger;
pub mod test_tcp;
pub mod wan_buf_rw;
use std::env;

pub struct ThreadPool {
    //handlers: Vec<thread::JoinHandle<()>>,
}

trait Hello {
    fn say_hi(&self);
}

struct TestHello {
    v: u16,
}

impl Default for TestHello {
    fn default() -> Self {
        TestHello { v: 123 }
    }
}

impl Hello for TestHello {
    fn say_hi(&self) {
        println!("say_hi:{}", self.v)
    }
}

impl TestHello {}

struct TestTrait<T> {
    hello: Box<T>,
}

impl<T> TestTrait<T>
where
    T: Hello + Default,
{
    pub fn new() -> Self {
        TestTrait {
            hello: Box::new(T::default()),
        }
    }
}

fn main() {
    /*
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    println!("Tevn is {} ", env!("PATH"));

    let th = Box::new(TestHello::default());

    let t: TestTrait<TestHello> = TestTrait::new();

    t.hello.say_hi();

    let xxx = vec![0u8; 10];
    */

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
