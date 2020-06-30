/*
use log::{error, info, warn};
use socket::clients;
use socket::clients::Client;
use socket::clients::Clients;
use socket::clients::ReadResult;
use socket::clients::WriteResult;
use socket::epoll::Epoll;
use socket::message::MsgData;
use socket::tcp_event::TcpEvent;
use socket::tcp_listen::TcpListen;
use std::net::Shutdown;
use std::net::TcpStream;
use std::thread;
*/
pub fn test() {
    /*
    client();
    client();
    client();
    client();
    client();
    client();

    match server() {
        Ok(joinhandle) => joinhandle.join().unwrap(),
        Err(err) => error!("{}", err),
    }]
    */
}
/*
fn write(client: &mut Client) -> bool {
    loop {
        match client.tcp_writer.write(&mut client.stream) {
            WriteResult::Finish => {
                //info!("write result:{}", "Finish");
                //thread::sleep(std::time::Duration::from_millis(1));
                return true; //break;
            }
            WriteResult::BufferFull => {
                error!("write result:{}", "BufferFull");
                thread::sleep(std::time::Duration::from_millis(10));
                //break;
            }
            WriteResult::Error(err) => {
                error!("write result error:{}", err);
                return false; //break;
            }
        }
    }
}

fn read(client: &mut Client) {
    loop {
        match client.tcp_reader.read(&mut client.stream) {
            ReadResult::Data(msg_data) => {
                info!(
                    "id:{} data:{:?}",
                    &msg_data.id,
                    String::from_utf8_lossy(&msg_data.data).to_string()
                );
                break;
                //return ReadResult::Data(msg_data);
            }

            ReadResult::BufferIsEmpty => {
                info!(
                    "read({:?})  BufferIsEmpty",
                    client.stream.local_addr().unwrap()
                );
                thread::sleep(std::time::Duration::from_millis(100));
                //return ReadResult::BufferIsEmpty;
            }
            ReadResult::ReadZeroSize => {
                error!(
                    "read({:?}) ReadZeroSize",
                    client.stream.local_addr().unwrap()
                );
                if let Err(err) = client.stream.shutdown(Shutdown::Both) {
                    error!("shutdown Error:{}", err);
                }
                break;
                //return ReadResult::ReadZeroSize;
            }
            ReadResult::MsgSizeTooBig => {
                error!(
                    "read({:?}) MsgSizeTooBig",
                    client.stream.local_addr().unwrap()
                );
                if let Err(err) = client.stream.shutdown(Shutdown::Both) {
                    error!("shutdown Error:{}", err);
                }

                break;
                //return ReadResult::MsgSizeTooBig;
            }
            ReadResult::MsgIdError => {
                error!(
                    "read({:?}) MsgPackIdError",
                    client.stream.local_addr().unwrap()
                );
                if let Err(err) = client.stream.shutdown(Shutdown::Both) {
                    error!("shutdown Error:{}", err);
                }
                break;
                //return ReadResult::MsgPackIdError;
            }
            ReadResult::Error(err) => {
                error!(
                    "read({:?}) error:{}",
                    client.stream.local_addr().unwrap(),
                    err
                );
                if let Err(err) = client.stream.shutdown(Shutdown::Both) {
                    error!("shutdown Error:{}", err);
                }
                break;
                //return ReadResult::Error(err);
            }
        }
    }
}

fn client() -> thread::JoinHandle<()> {
    std::thread::spawn(move || {
        info!("client-->{:?}", std::thread::current().id());
        thread::sleep(std::time::Duration::from_secs(5));

        match TcpStream::connect("0.0.0.0:9988") {
            Ok(tcp_strem) => {
                warn!("connect success:{:?}", tcp_strem.local_addr());

                let mut client = Client::new(tcp_strem, 1024);

                let str = "0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz";

                let buffer = str.as_bytes();

                thread::sleep(std::time::Duration::from_secs(1));
                let mut msg_num: u64 = 0;

                let mut msg_len = 10;

                loop {
                    msg_num += 1;

                    msg_len += 1;
                    if msg_len == str.len() {
                        msg_len = 10;
                    }

                    //thread::sleep(std::time::Duration::from_secs(1));
                    /*
                    info!(
                        "write new data start:{}---------------------------",
                        msg_num
                    );
                    */
                    let mut data: Vec<u8> = vec![0u8; msg_len];
                    data.copy_from_slice(&buffer[0..msg_len]);

                    let msg_data = Box::new(MsgData { id: 1, data: data });

                    if let Err(err) = client.tcp_writer.add_msg_data(msg_data) {
                        info!("add_msg_data result err:{}", err);
                    }

                    if write(&mut client) == false {
                        break;
                    }

                    if msg_num % 10000000 == 0 {
                        info!("write data:{}", msg_num);
                    }

                    //read(&mut client);
                }
            }

            Err(err) => error!("connect error:{}", err),
        }
    })
}

fn server() -> Result<thread::JoinHandle<()>, String> {
    let epoll: Epoll;
    match Epoll::new() {
        Ok(ep) => epoll = ep,
        Err(err) => {
            return Err(err);
        }
    };

    let server = thread::spawn(move || match Clients::new(99, 1024, &epoll) {
        clients::NewClientsResult::Ok(mut clients) => {
            info!("server-->{:?}", thread::current().name());

            info!("server-->{:?}", thread::current().id());

            let mut epevent = TcpEvent::new(&epoll, &mut clients);
            match TcpListen::new(&String::from("0.0.0.0:9988"), 99, &epoll, &mut epevent) {
                Ok(mut listen) => loop {
                    match listen.run(1) {
                        Ok(()) => (),
                        Err(err) => error!("{}", err),
                    }
                },
                Err(err) => error!("tcplisten:new error:{}", err),
            }
        }
        clients::NewClientsResult::MsgSizeTooBig => {
            error!("Clients::new error:{:?}", "MsgSizeTooBig")
        }
        clients::NewClientsResult::ClientNumTooSmall => {
            error!("Clients::new error:{:?}", "ClientNumTooSmall")
        }
    });

    Ok(server)
}
*/
