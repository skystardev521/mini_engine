//use std::thread;
use utils::logger;
//use utils::time;
pub mod test_tcp;

pub struct ThreadPool {
    //handlers: Vec<thread::JoinHandle<()>>,
}

fn main() {
    match logger::Logger::init(&String::from("info"), &String::from("logs/test_log.log"), 1) {
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

    let mut hm: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();

    let mut f_mut = || -> u16 {
        hm.insert("key", 999);
        9999
    };

    fn_mut_closure(&mut f_mut);
    fn_mut_closure_1(&mut f_mut);

    fn_mut_closure(&mut || -> u16 {
        hm.insert("key", 999);
        return 9999;
    });

    let f = || -> u16 { 9999 };

    fn_closure(&f);
    fn_closure_1(f);

    let fp = |p| -> u16 { 9999 + p };

    fn_pram(fp);
    fn_pram_1(fp);
    fn_once_closure_1(f);

    let _ret_clos = returns_closure();
}

fn fn_pram<F>(f: F)
where
    F: Fn(u16) -> u16,
{
    println!("f result:{}", f(123));
}

fn fn_pram_1(f: fn(u16) -> u16) {
    println!("f result:{}", f(123));
}

fn fn_mut_closure(f: &mut dyn FnMut() -> u16) {
    f();
}
fn fn_mut_closure_1<F>(f: &mut F)
where
    F: FnMut() -> u16,
{
    f();
}
/*
fn fn_mut_closure_2<F>(f: &mut F)
where
    F: FnMut(u16) -> u16,
{
    f(1);
}
*/
fn fn_closure(f: &dyn Fn() -> u16) {
    f();
}
fn fn_closure_1<F>(f: F)
where
    F: Fn() -> u16,
{
    f();
}

fn fn_once_closure_1<F>(f: F)
where
    F: FnOnce() -> u16,
{
    f();
}

fn returns_closure() -> Box<dyn Fn(i32) -> i32> {
    Box::new(|x| x + 1)
}
