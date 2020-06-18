use socket::clients::Clients;
use socket::tcp_event::TcpEvent;
use socket::tcp_listen::TcpListen;

#[derive(Debug)]
enum TestEnum {
    Ok,
    Err,
}

fn main() {
    let server = std::thread::spawn(move || match Clients::new(99, 1024) {
        Ok(mut clients) => {
            let tcp_event = TcpEvent::new(&mut clients);
            match TcpListen::new(1, 99, &String::from("0.0.0.0:9988"), tcp_event) {
                Ok(mut listen) => {
                    loop{
                        match listen.wait_events(1000){
                            Ok(())=>(),
                            Err(err)=> println!("wait_events:{}", err)
                        }
                    }
                }
                Err(err) => println!("tcplisten:new error:{}", err),
            }
        }
        Err(err) => { println!("Clients::new error:{:?}", err)}
    });

    let client = std::thread::spawn(move || {});

    match server.join() {
        Ok(())=> println!("server.join ok"),
        Err(err)=>println!("server.join:{:?}", err)
    }

    let (tx, rx) = std::sync::mpsc::channel();

    let child_thread = std::thread::spawn(move || {
        tx.send(TestEnum::Err).unwrap();
    });

    child_thread.join().unwrap();

    let received = rx.recv().unwrap();
    println!("Got: {:?}", received);

    println!("TestEnum:{:?}", TestEnum::Err);

    let mut hm: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
    let mut f_mut = || -> u16 {
        hm.insert("key", 999);
        9999
    };
    fn_mut_closure(&mut f_mut);
    fn_mut_closure_1(&mut f_mut);

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
