use log::{error, info, warn};

use mini_socket::tcp_socket::TcpSocket;
use mini_socket::tcp_socket_rw::ReadResult;
use mini_socket::tcp_socket_rw::WriteResult;
use mini_socket::tcp_socket_msg::MsgData;
use mini_utils::bytes;

use std::net::Shutdown;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use mini_utils::time;

use crate::wan_tcp_rw::WanTcpRw;

const LOG_FILE_DURATION: u64 = 60 * 60 * 1000;

pub fn test() {
    let mut open_log_file_ts = time::timestamp();

    let mut thread_pool: Vec<(thread::JoinHandle<()>, thread::JoinHandle<()>)> = Vec::new();

    thread::sleep(std::time::Duration::from_secs(1));

    for _ in 0..1{
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
        let wan_tcp_rw = Box::new(WanTcpRw::default());
        let mut client = TcpSocket::new(socket, wan_tcp_rw);
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
            if msg_num % 100 == 0 {
                info!("write data:{} {:?}", msg_num, thread::current().id());
            }
            if msg_num % 100 == 0{
                thread::sleep(std::time::Duration::from_millis(1));
            }
            if write(&mut client) == false {
                break;
            }
        }
    })
}

fn encode(ext: u32) -> MsgData {
    let str = "0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz";
    let rlen: usize = 100 + (ext % 300) as usize;
    let mut buf = vec![0u8; rlen];
    bytes::write_bytes(&mut buf, &str.as_bytes()[0..rlen]);
    MsgData{
        uid: 0,
        pid: 258,
        ext: ext,
        buf: buf
    }
}

fn write(client: &mut TcpSocket<MsgData>) -> bool {
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
        let wan_tcp_rw = Box::new(WanTcpRw::default());
        let mut client = TcpSocket::new(socket, wan_tcp_rw);
        if read(&mut client) == false {
            break;
        }
    })
}

fn read(client: &mut TcpSocket<MsgData>) -> bool {
    let mut share_buffer = vec![0u8; 1024 * 1024];
    loop {
        match client.read(&mut share_buffer) {
            ReadResult::Data(vec_msg) => {
                if vec_msg.is_empty(){
                    thread::sleep(std::time::Duration::from_millis(1));
                }
                for msg in vec_msg.iter(){
                    if msg.ext % 10000 == 0 {
                        info!("read ext data:{} {:?}", msg.ext, thread::current().id());
                    }
                }
            }
            ReadResult::Error(vec_msg, err) => {
                for msg in vec_msg.iter(){
                    if msg.ext % 10000 == 0 {
                        info!("read ext data:{} {:?}", msg.ext, thread::current().id());
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
