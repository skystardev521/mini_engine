use log::{error, info, warn};

use mini_socket::tcp_buf_rw::ReadResult;
use mini_socket::tcp_buf_rw::WriteResult;
use mini_socket::tcp_socket::TcpSocket;
use mini_utils::bytes;

use std::net::Shutdown;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use mini_utils::time;

use crate::wan_buf_rw::WanBufRw;

const LOG_FILE_DURATION: u64 = 60 * 60 * 1000;

pub fn test() {
    let mut open_log_file_ts = time::timestamp();

    let mut thread_pool: Vec<(thread::JoinHandle<()>, thread::JoinHandle<()>)> = Vec::new();

    thread::sleep(std::time::Duration::from_secs(1));

    for _ in 0..1 {
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
    thread::spawn(move || {
        let wan_buf_rw = Box::new(WanBufRw::default());
        let mut client = TcpSocket::new(socket, wan_buf_rw);
        info!("client-->{:?}", std::thread::current().id());
        let mut msg_num: u64 = 0;

        let mut ext_data: u32 = 0;
        loop {
            ext_data += 1;
            if ext_data == u32::MAX {
                ext_data = 0;
            }
            let msg_data = encode(ext_data);
            client.push_vec_queue(msg_data);
            msg_num += 1;
            if msg_num % 10000 == 0 {
                info!("write data:{} {:?}", msg_num, thread::current().id());
            }
            //if msg_num % 10 == 0{
                thread::sleep(std::time::Duration::from_millis(1));
            //}
            if write(&mut client) == false {
                break;
            }
        }
    })
}

fn encode(ext: u32) -> Vec<u8> {
    let str = "0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz";
    let rlen: usize = 100 + (ext % 300) as usize;
    let len = 2 + 4 + rlen;
    let mut buffer: Vec<u8> = vec![0u8; len];
    bytes::write_u16(&mut buffer, 123);
    bytes::write_u32(&mut buffer[2..], ext);
    bytes::write_bytes(&mut buffer[6..], &str.as_bytes()[0..rlen]);
    //warn!("encode buffer len:{} ext:{}", buffer.len(), ext);
    buffer
}

fn decode(buffer: &Vec<u8>) -> (u16, u32, Vec<u8>) {
    //warn!("decode buffer len:{}", buffer.len());
    let pid = bytes::read_u16(&buffer);
    let ext = bytes::read_u32(&buffer[2..]);
    let data = bytes::read_bytes(&buffer[6..]);
    (pid, ext, data)
}

fn write(client: &mut TcpSocket<Vec<u8>>) -> bool {
    //let mut msg_num: u64 = 0;
    loop {
        match client.write() {
            WriteResult::Finish => {
                //println!("Finish:{}", mini_utils::time::timestamp());
                
                return true;
            }
            WriteResult::BufferFull => {
                //thread::sleep(std::time::Duration::from_millis(1));
                //return true;
                println!("BufferFull:{}", mini_utils::time::timestamp());
            }
            WriteResult::Error(err) => {
                error!("write result error:{}", err);
                return false; //break;
            }
        }
    }
}

fn loop_read(socket: TcpStream) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let socket = socket.try_clone().unwrap();
        let wan_buf_rw = Box::new(WanBufRw::default());
        let mut client = TcpSocket::new(socket, wan_buf_rw);
        if read(&mut client) == false {
            break;
        }
    })
}

fn read(client: &mut TcpSocket<Vec<u8>>) -> bool {
    let mut vec_share = vec![0u8; 1024 * 1024];
    loop {
        match client.read(&mut vec_share) {
            ReadResult::Data(vec_msg) => {
                for msg in vec_msg {
                    let (pid, ext, data) = decode(&msg);
                    /*
                    info!(
                        "read pid:{} ext:{} data:{}",
                        pid,
                        ext,
                        String::from_utf8_lossy(&data).to_string()
                    );
                    */
                    if ext % 10000 == 0 {
                        info!("read ext data:{} {:?}", ext, thread::current().id());
                    }
                }
            }
            ReadResult::Error(vec_msg, err) => {
                for msg in vec_msg {
                    let (pid, ext, data) = decode(&msg);
                    if ext % 10000 == 0 {
                        info!("read ext data:{} {:?}", ext, thread::current().id());
                    }
                }
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
