use crate::tcp_connect::TcpConnect;
use crate::tcp_connect_config::TcpConnectConfig;
use crate::tcp_socket::TcpSocket;
use mini_utils::time;
use std::collections::HashMap;
use std::net::TcpStream;
use std::time::Duration;

pub struct TcpConnectMgmt<'a> {
    //不会等于零
    next_sid: u64,
    config: &'a TcpConnectConfig,
    /// 可以优化使用别的数据结构 列如 vec
    tcp_connect_hash_map: HashMap<u64, TcpConnect>,
}

impl<'a> TcpConnectMgmt<'a> {
    pub fn new(config: &'a TcpConnectConfig) -> Result<Self, String> {
        Ok(TcpConnectMgmt {
            next_sid: 0,
            config: config,
            tcp_connect_hash_map: HashMap::new(),
        })
    }

    fn next_sid(&self) -> u64 {
        let mut sid = self.next_sid;
        loop {
            sid += 1;
            if sid == 0 {
                sid = 1;
            }
            if self.tcp_connect_hash_map.contains_key(&sid) {
                continue;
            }
            return sid;
        }
    }

    #[inline]
    pub fn get_connect_count(&self) -> u32 {
        self.tcp_connect_hash_map.len() as u32
    }

    #[inline]
    pub fn get_tcp_connect(&mut self, sid: u64) -> Option<&mut TcpConnect> {
        self.tcp_connect_hash_map.get_mut(&sid)
    }

    #[inline]
    pub fn del_socket(&mut self, sid: u64) -> Result<TcpConnect, String> {
        if let Some(tcp_connect) = self.tcp_connect_hash_map.remove(&sid) {
            Ok(tcp_connect)
        } else {
            Err(format!("del_client id:{} not exists", sid))
        }
    }

    pub fn add_tcp_connect(
        &mut self,
        tcp_socket: TcpSocket,
        socket_addr: &String,
    ) -> Result<u64, String> {
        /*
        let tcp_socket = new_tcp_socket(
            socket_addr,
            self.config.msg_max_size,
            self.config.connect_timeout_duration,
        )?;
        */

        self.next_sid = self.next_sid();
        self.tcp_connect_hash_map.insert(
            self.next_sid,
            TcpConnect::new(self.next_sid, socket_addr, Some(tcp_socket)),
        );
        Ok(self.next_sid)
    }

    /*
    #[inline]
    fn reconnect_tcp_connect(
        config: &TcpClientConfig,
        tcp_connect: &TcpConnect,
    ) -> Result<Option<TcpSocket>, String> {
        let now_timestamp = time::timestamp();
        if tcp_client.get_last_connect_timestamp() + config.reconnect_socket_interval as u64
            > now_timestamp
        {
            return Ok(None);
        }
        match new_tcp_socket(
            tcp_connect.get_socket_addr(),
            config.msg_max_size,
            config.connect_timeout_duration,
        ) {
            Ok(tcp_connect) => return Ok(Some(tcp_connect)),
            Err(err) => return Err(err),
        }
    }
    */
}

/*
use crate::message::NetMsg;
use crate::tcp_client::TcpClient;
use crate::tcp_client_config::TcpClientConfig;
use crate::tcp_connect::TcpSocket;
use log::{error, warn};
// std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;
use mini_utils::time;

use crate::tcp_connect::ReadResult;
use crate::tcp_connect::WriteResult;

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
            if let Some(tcp_connect) = tcp_client.get_tcp_connect_opt() {
                match read(id, tcp_connect, self.net_msg_cb) {
                    Ok(()) => (),
                    Err(err) => {
                        error!("{}", err);
                        tcp_client.set_tcp_connect_opt(None);
                        continue;
                    }
                }
                match write(tcp_connect) {
                    Ok(()) => (),
                    Err(err) => {
                        error!("{}", err);
                        tcp_client.set_tcp_connect_opt(None);
                    }
                }
            } else {
                match reconnect_tcp_connect(self.config, &tcp_client) {
                    Ok(Some(mut tcp_connect)) => {
                        tcp_connect.reader.read(&mut tcp_connect.socket);

                        //tcp_client.set_tcp_connect_opt(Some(tcp_connect1));
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
    pub fn get_tcp_connect(&mut self, id: u8) -> Option<&mut TcpClient> {
        self.vec_tcp_client.get_mut(id as usize)
    }

    #[inline]
    pub fn del_socket(&mut self, id: u8) -> Result<(), String> {
        if (id as usize) < self.vec_tcp_client.len() {
            self.vec_tcp_client.remove(id as usize);
            Ok(())
        } else {
            return Err("id >= self.vec_tcp_connect.len()".into());
        }
    }
    */
    pub fn new_tcp_client(&mut self, socket_addr: &String) -> Result<TcpClient, String> {
        match new_tcp_connect(
            socket_addr,
            self.config.msg_max_size,
            self.config.connect_timeout_duration,
        ) {
            Ok(tcp_connect) => {
                let id = self.vec_tcp_client.len() as u8;
                Ok(TcpClient::new(id, socket_addr, Some(tcp_connect)))
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

fn new_tcp_connect(
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
                        return Err(err.to_string());
                    }
                    return Ok(TcpSocket::new(socket, msg_max_size));
                }
                Err(err) => return Err(err.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[inline]
fn reconnect_tcp_connect(
    config: &TcpClientConfig,
    tcp_client: &TcpClient,
) -> Result<Option<TcpSocket>, String> {
    let now_timestamp = time::timestamp();
    if tcp_client.get_last_connect_timestamp() + config.reconnect_socket_interval as u64
        > now_timestamp
    {
        return Ok(None);
    }
    match new_tcp_connect(
        tcp_client.get_socket_addr(),
        config.msg_max_size,
        config.connect_timeout_duration,
    ) {
        Ok(tcp_connect) => return Ok(Some(tcp_connect)),
        Err(err) => return Err(err),
    }
}

fn read(
    net_msg_id: u8,
    tcp_connect: &mut TcpSocket,
    net_msg_cb: &mut dyn Fn(NetMsg),
) -> Result<(), String> {
    loop {
        match tcp_connect.reader.read(&mut tcp_connect.socket) {
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
fn write(tcp_connect: &mut TcpSocket) -> Result<(), String> {
    match tcp_connect.writer.write(&mut tcp_connect.socket) {
        WriteResult::Finish => Ok(()),
        WriteResult::BufferFull => Ok(()),
        WriteResult::Error(err) => Err(err),
    }
}
*/
