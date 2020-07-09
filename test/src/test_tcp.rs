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
    let mut thread_pool: Vec<(thread::JoinHandle<()>, thread::JoinHandle<()>)> = Vec::new();

    thread::sleep(std::time::Duration::from_secs(1));

    for _ in 0..300 {
        thread_pool.push(new_client().unwrap());
    }

    loop {
        thread::sleep(Duration::from_secs(60))
    }
}

fn loop_write(socket: TcpStream) -> thread::JoinHandle<()> {
    let mut client = TcpSocket::new(socket, 2024);
    thread::spawn(move || {
        info!("client-->{:?}", std::thread::current().id());
        let str = "0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz";

        let buffer = str.as_bytes();
        let mut msg_len = 100;
        let mut msg_num: u64 = 0;
        loop {
            msg_len += 1;
            if msg_len == str.len() {
                msg_len = 100;
            }
            let mut data: Vec<u8> = vec![0u8; msg_len];
            data.copy_from_slice(&buffer[0..msg_len]);

            let msg_data = Box::new(MsgData {
                pid: 1,
                ext: 0,
                data: data,
            });

            if let Err(err) = client.writer.add_msg_data(msg_data) {
                info!("add_msg_data result err:{}", err);
            }
            msg_num += 1;
            if msg_num % 1000 == 0 {
                info!("write data:{}", msg_num);
            }
            write(&mut client);
        }
    })
}

fn write(client: &mut TcpSocket) {
    //let mut msg_num: u64 = 0;
    loop {
        match client.writer.write(&mut client.socket) {
            WriteResult::Finish => {
                //info!("write result:Finish");
                thread::sleep(std::time::Duration::from_millis(5));
                //return true; //

                //info!("write data:{}", msg_num);
                break;
            }
            WriteResult::BufferFull => {
                //error!("write result:{}", "BufferFull");
                //thread::sleep(std::time::Duration::from_millis(1));
                break;
            }
            WriteResult::Error(err) => {
                error!("write result error:{}", err);
                //return false; //break;
            }
        }
    }
}

fn loop_read(socket: TcpStream) -> thread::JoinHandle<()> {
    let mut client = TcpSocket::new(socket, 1024);
    thread::spawn(move || loop {
        read(&mut client);
    })
}

fn read(client: &mut TcpSocket) {
    let mut msg_num: u64 = 0;
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
                msg_num += 1;
                if msg_num % 1000 == 0 {
                    info!("read data:{}", msg_num);
                }
                //break;
            }

            ReadResult::BufferIsEmpty => {
                /*
                info!(
                    "read({:?})  BufferIsEmpty",
                    client.socket.local_addr().unwrap()
                );
                */
                //thread::sleep(std::time::Duration::from_millis(1));
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
            }
        }
    }
}

fn new_client() -> Result<(thread::JoinHandle<()>, thread::JoinHandle<()>), String> {
    match TcpStream::connect("0.0.0.0:9999") {
        Ok(socket) => {
            warn!("connect success:{:?}", socket.local_addr());

            let loop_write_thread = loop_write(socket.try_clone().unwrap());
            let loop_read_thread = loop_read(socket.try_clone().unwrap());

            return Ok((loop_write_thread, loop_read_thread));
        }

        Err(err) => {
            error!("connect error:{}", err);
            return Err(format!("{}", err));
        }
    }
}
