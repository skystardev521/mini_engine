use log::{error, info, warn};
use mini_socket::tcp_socket::TcpSocket;
use mini_socket::tcp_socket_reader::ReadResult;
use mini_socket::tcp_socket_writer::WriteResult;
use mini_utils::bytes;

use std::net::Shutdown;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use mini_utils::time;

const LOG_FILE_DURATION: u64 = 60 * 60 * 1000;

pub fn test() {
    let mut open_log_file_ts = time::timestamp();
    let mut thread_pool: Vec<(thread::JoinHandle<()>, thread::JoinHandle<()>)> = Vec::new();

    thread::sleep(std::time::Duration::from_secs(1));

    for _ in 0..300 {
        thread_pool.push(new_client().unwrap());
    }

    loop {
        thread::sleep(Duration::from_secs(60));
        if open_log_file_ts + LOG_FILE_DURATION < time::timestamp() {
            log::logger().flush();
            open_log_file_ts = time::timestamp();
        }
    }
}

fn loop_write(socket: TcpStream) -> thread::JoinHandle<()> {
    let mut client = TcpSocket::new(socket, 10240);
    thread::spawn(move || {
        info!("client-->{:?}", std::thread::current().id());
        let mut msg_num: u64 = 0;

        let mut ext_data: u32 = 0;
        loop {
            ext_data += 1;
            let msg_data = encode(ext_data);

            if let Err(err) = client.writer.add_msg_data(msg_data) {
                info!("add_msg_data result err:{}", err);
            }
            msg_num += 1;
            if msg_num % 10000 == 0 {
                info!("write data:{}", msg_num);
            }
            if write(&mut client) == false {
                break;
            }
        }
    })
}

fn encode(ext: u32) -> Vec<u8> {
    let str = "0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz";
    let len = 2 + 4 + 512;
    let mut buffer: Vec<u8> = vec![0u8; len];
    bytes::write_u16(&mut buffer, 123);
    bytes::write_u32(&mut buffer, ext);
    bytes::write_bytes(&mut buffer, &str.as_bytes()[0..512]);
    buffer
}

fn decode(buffer: &Vec<u8>) -> (u16, u32, Vec<u8>) {
    let pid = bytes::read_u16(&buffer);
    let ext = bytes::read_u32(&buffer[2..]);
    let data = bytes::read_bytes(&buffer[6..]);
    (pid, ext, data)
}

fn write(client: &mut TcpSocket) -> bool {
    //let mut msg_num: u64 = 0;
    loop {
        match client.writer.write(&mut client.socket) {
            WriteResult::Finish => {
                //info!("write result:Finish");
                thread::sleep(std::time::Duration::from_millis(10));
                //return true; //

                //info!("write data:{}", msg_num);
                return true;
            }
            WriteResult::BufferFull => {
                //error!("write result:{}", "BufferFull");
                //thread::sleep(std::time::Duration::from_millis(1));
                return true;
            }
            WriteResult::Error(err) => {
                error!("write result error:{}", err);
                return false; //break;
            }
        }
    }
}

fn loop_read(socket: TcpStream) -> thread::JoinHandle<()> {
    let mut client = TcpSocket::new(socket, 10240);
    thread::spawn(move || loop {
        if read(&mut client) == false {
            break;
        }
    })
}

fn read(client: &mut TcpSocket) -> bool {
    let mut msg_num: u64 = 0;
    loop {
        match client.reader.read(&mut client.socket) {
            ReadResult::Data(msg_data) => {
                let (pid, ext, data) = decode(&msg_data);
                /*
                info!(
                    "read pid:{} ext:{} data:{}",
                    pid,
                    ext,
                    String::from_utf8_lossy(&data).to_string()
                );
                */
                msg_num += 1;
                if msg_num % 10000 == 0 {
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
                return false;
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
                return false;
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
            return Err(err.to_string());
        }
    }
}
