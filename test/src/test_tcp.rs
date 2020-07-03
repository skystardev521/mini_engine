use log::{error, info, warn};

use socket::message::MsgData;
use socket::tcp_socket::ReadResult;
use socket::tcp_socket::TcpSocket;
use socket::tcp_socket::WriteResult;
use std::net::Shutdown;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub fn test() {
    for _ in 0..10000 {
        client();
    }

    loop {
        thread::sleep(Duration::from_secs(60))
    }
    /*
    match server() {
        Ok(joinhandle) => joinhandle.join().unwrap(),
        Err(err) => error!("{}", err),
    }
    */
}

fn write(client: &mut TcpSocket) -> bool {
    loop {
        match client.writer.write(&mut client.socket) {
            WriteResult::Finish => {
                //info!("write result:{}", "Finish");
                //thread::sleep(std::time::Duration::from_millis(1));
                return true; //break;
            }
            WriteResult::BufferFull => {
                //error!("write result:{}", "BufferFull");
                //thread::sleep(std::time::Duration::from_millis(1));
                //break;
            }
            WriteResult::Error(err) => {
                error!("write result error:{}", err);
                return false; //break;
            }
        }
    }
}

fn read(client: &mut TcpSocket) {
    loop {
        match client.reader.read(&mut client.socket) {
            ReadResult::Data(msg_data) => {
                /*
                info!(
                    "read id:{} data:{:?}",
                    &msg_data.id,
                    String::from_utf8_lossy(&msg_data.data).to_string()
                );
                */
                break;
            }

            ReadResult::BufferIsEmpty => {
                info!(
                    "read({:?})  BufferIsEmpty",
                    client.socket.local_addr().unwrap()
                );
                //thread::sleep(std::time::Duration::from_millis(1));
                //return ReadResult::BufferIsEmpty;
            }
            ReadResult::ReadZeroSize => {
                error!(
                    "read({:?}) ReadZeroSize",
                    client.socket.local_addr().unwrap()
                );
                if let Err(err) = client.socket.shutdown(Shutdown::Both) {
                    error!("shutdown Error:{}", err);
                }
                break;
            }
            ReadResult::Error(err) => {
                error!(
                    "read({:?}) error:{}",
                    client.socket.local_addr().unwrap(),
                    err
                );
                if let Err(err) = client.socket.shutdown(Shutdown::Both) {
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

        match TcpStream::connect("0.0.0.0:9999") {
            Ok(tcp_strem) => {
                warn!("connect success:{:?}", tcp_strem.local_addr());

                let mut client = TcpSocket::new(tcp_strem, 1024);

                let str = "0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz";

                let buffer = str.as_bytes();

                thread::sleep(std::time::Duration::from_secs(1));
                let mut msg_num: u64 = 0;

                let mut msg_len = 10;

                loop {
                    msg_num += 1;

                    msg_len += 1;
                    if msg_len == str.len() {
                        msg_len = 32;
                    }

                    //thread::sleep(std::time::Duration::from_millis(1));

                    let mut data: Vec<u8> = vec![0u8; msg_len];
                    data.copy_from_slice(&buffer[0..msg_len]);

                    let msg_data = Box::new(MsgData { id: 1, data: data });

                    if let Err(err) = client.writer.add_msg_data(msg_data) {
                        info!("add_msg_data result err:{}", err);
                    }

                    if write(&mut client) == false {
                        break;
                    }

                    if msg_num % 10000000 == 0 {
                        info!("read write data:{}", msg_num);
                    }

                    read(&mut client);
                }
            }

            Err(err) => error!("connect error:{}", err),
        }
    })
}
