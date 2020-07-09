/*
use crate::message::NetMsg;
use crate::tcp_client::TcpClient;
use crate::tcp_client_config::TcpClientConfig;
use crate::tcp_socket::TcpSocket;
use log::{error, warn};
// std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;
use utils::time;

use crate::tcp_socket::ReadResult;
use crate::tcp_socket::WriteResult;

pub struct TcpClientMgmt<'a> {
    config: &'a TcpClientConfig,
    vec_tcp_client: Vec<TcpClient>,
    net_msg_cb: &'a mut dyn Fn(NetMsg),
}

impl<'a> TcpClientMgmt<'a> {
    pub fn new(config: &'a TcpClientConfig, net_msg_cb: &'a mut dyn Fn(NetMsg)) -> Self {
        TcpClientMgmt {
            config,
            net_msg_cb,
            vec_tcp_client: Vec::with_capacity(config.vec_socket_addr.len()),
        }
    }

    pub fn connect_all(&mut self) -> Result<(), String> {
        if self.vec_tcp_client.len() > 0 {
            return Err("repeat call connect_all".into());
        }
        let mut vec_str: Vec<String> = vec![];
        for addr in &self.config.vec_socket_addr {
            let id = self.vec_tcp_client.len() as u8;
            match self.new_tcp_client(&addr) {
                Ok(tcp_client) => {
                    self.vec_tcp_client.push(tcp_client);
                }
                Err(err) => {
                    vec_str.push(format!("connect:{} error:{}\n", addr, err));
                    self.vec_tcp_client.push(TcpClient::new(id, &addr, None));
                }
            }
        }
        if vec_str.len() == 0 {
            Ok(())
        } else {
            Err(vec_str.concat())
        }
    }

    pub fn tick(&mut self) {
        for tcp_client in self.vec_tcp_client.iter_mut() {
            let id = tcp_client.get_id();
            if let Some(tcp_socket) = tcp_client.get_tcp_socket_opt() {
                match read(id, tcp_socket, self.net_msg_cb) {
                    Ok(()) => (),
                    Err(err) => {
                        error!("{}", err);
                        tcp_client.set_tcp_socket_opt(None);
                        continue;
                    }
                }
                match write(tcp_socket) {
                    Ok(()) => (),
                    Err(err) => {
                        error!("{}", err);
                        tcp_client.set_tcp_socket_opt(None);
                    }
                }
            } else {
                match reconnect_tcp_socket(self.config, &tcp_client) {
                    Ok(Some(mut tcp_socket)) => {
                        tcp_socket.reader.read(&mut tcp_socket.socket);

                        //tcp_client.set_tcp_socket_opt(Some(tcp_socket1));
                    }
                    Ok(None) => continue,
                    Err(err) => {
                        error!("{}", err);
                        continue;
                    }
                }
            }
        }
    }
    /*
    #[inline]
    pub fn get_tcp_socket(&mut self, id: u8) -> Option<&mut TcpClient> {
        self.vec_tcp_client.get_mut(id as usize)
    }

    #[inline]
    pub fn del_socket(&mut self, id: u8) -> Result<(), String> {
        if (id as usize) < self.vec_tcp_client.len() {
            self.vec_tcp_client.remove(id as usize);
            Ok(())
        } else {
            return Err("id >= self.vec_tcp_socket.len()".into());
        }
    }
    */
    pub fn new_tcp_client(&mut self, socket_addr: &String) -> Result<TcpClient, String> {
        match new_tcp_socket(
            socket_addr,
            self.config.msg_max_size,
            self.config.connect_timeout_duration,
        ) {
            Ok(tcp_socket) => {
                let id = self.vec_tcp_client.len() as u8;
                Ok(TcpClient::new(id, socket_addr, Some(tcp_socket)))
            }
            Err(err) => Err(format!("{}", err)),
        }
    }
}

fn new_tcp_socket(
    socket_addr: &String,
    msg_max_size: u32,
    timeout_duration: u16,
) -> Result<TcpSocket, String> {
    match socket_addr.parse::<SocketAddr>() {
        Ok(addr) => {
            let duration = Duration::from_millis(timeout_duration as u64);
            match TcpStream::connect_timeout(&addr, duration) {
                Ok(socket) => {
                    if let Err(err) = socket.set_nonblocking(true) {
                        return Err(format!("{}", err));
                    }
                    return Ok(TcpSocket::new(socket, msg_max_size));
                }
                Err(err) => return Err(format!("{}", err)),
            }
        }
        Err(err) => Err(format!("{}", err)),
    }
}

#[inline]
fn reconnect_tcp_socket(
    config: &TcpClientConfig,
    tcp_client: &TcpClient,
) -> Result<Option<TcpSocket>, String> {
    let now_timestamp = time::timestamp();
    if tcp_client.get_last_connect_timestamp() + config.reconnect_socket_interval as u64
        > now_timestamp
    {
        return Ok(None);
    }
    match new_tcp_socket(
        tcp_client.get_socket_addr(),
        config.msg_max_size,
        config.connect_timeout_duration,
    ) {
        Ok(tcp_socket) => return Ok(Some(tcp_socket)),
        Err(err) => return Err(err),
    }
}

fn read(
    net_msg_id: u8,
    tcp_socket: &mut TcpSocket,
    net_msg_cb: &mut dyn Fn(NetMsg),
) -> Result<(), String> {
    loop {
        match tcp_socket.reader.read(&mut tcp_socket.socket) {
            ReadResult::Data(msg_data) => {
                net_msg_cb(NetMsg {
                    id: net_msg_id as u64,
                    data: msg_data,
                });
            }
            ReadResult::BufferIsEmpty => return Ok(()),

            ReadResult::ReadZeroSize => {
                return Err("Read Zero Size".into());
            }
            ReadResult::Error(err) => return Err(err),
        }
    }
}

#[inline]
fn write(tcp_socket: &mut TcpSocket) -> Result<(), String> {
    match tcp_socket.writer.write(&mut tcp_socket.socket) {
        WriteResult::Finish => Ok(()),
        WriteResult::BufferFull => Ok(()),
        WriteResult::Error(err) => Err(err),
    }
}
*/
