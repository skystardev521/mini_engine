use crate::config::NetConfig;
use crate::logic::Logic;
//use crate::net::Net;
use log::{error, info};
use socket::clients::Clients;
use socket::clients::NewClientsResult;
use socket::epoll::Epoll;
use socket::message::NetMsg;
use socket::tcp_event::TcpEvent;
use socket::tcp_listen::TcpListen;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::Duration;

use socket::message::MsgData;

pub struct Engine {
    is_exit: bool,
}

impl Engine {
    /*
    pub fn init(config: &Config) -> Result<Self, String> {
        return OK(Engine { config: confg });
    }
    */

    pub fn run(&self) {
        let net_cfg = NetConfig {
            max_client: 1024,
            msg_max_size: 16 * 1024,
            epoll_max_events: 1024,
            epoll_wait_timeout: 1,
            tcp_linsten_addr: "0.0.0.0:9988".to_string(),
        };
        let channel_size = 10000;

        let net_builder = thread::Builder::new().name("Net".into()); //.stack_size(stack_size);
        let logic_builder = thread::Builder::new().name("Logic".into());

        //阻塞模式 设置队列大小
        let (net_syncsender, logic_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
            mpsc::sync_channel(channel_size);
        let (logic_syncsender, net_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
            mpsc::sync_channel(channel_size);

        let net_thread = net_builder.spawn(move || {
            let epoll: Epoll;
            match Epoll::new() {
                Ok(ep) => epoll = ep,
                Err(err) => {
                    error!("Epoll::new:{}", err);
                    return; //Err(err),
                }
            };

            let mut clients: Clients;
            match Clients::new(net_cfg.max_client, net_cfg.msg_max_size, &epoll) {
                NewClientsResult::Ok(cs) => clients = cs,
                NewClientsResult::MsgSizeTooBig => {
                    error!("Clients::new error:{}", "MsgSizeTooBig");
                    return; //Err(String::from("MsgSizeTooBig"));
                }
                NewClientsResult::ClientNumTooSmall => {
                    error!("Clients::new error:{}", "ClientNumTooSmall");
                    return; // Err(String::from("ClientNumTooSmall"));
                }
            };

            let mut msg_data_cb = |net_msg: NetMsg| {
                if let Err(err) = net_syncsender.send(net_msg) {
                    error!("net_syncsender.send error:{}", err);
                }
            };

            let mut tcp_event = TcpEvent::new(&epoll, &mut clients, &mut msg_data_cb);

            let mut tcp_listen: TcpListen;
            match TcpListen::new(
                &net_cfg.tcp_linsten_addr,
                net_cfg.epoll_max_events,
                &epoll,
                &mut tcp_event,
            ) {
                Ok(listen) => tcp_listen = listen,
                Err(err) => {
                    error!("TcpListen::new error:{}", err);
                    return; // Err(err);
                }
            }

            loop {
                if let Err(err) = tcp_listen.run(net_cfg.epoll_wait_timeout) {
                    error!("tcp_listen.run:{}", err);
                    break;
                }
            }
            /*
            loop {
                match net_receiver.try_recv() {
                    Ok(net_msg) => {
                        if let Some(client) = clients.get_mut_client(net_msg.id) {
                            //client.tcp_writer.add_msg_data()
                        }
                    }
                    Err(_err) => {
                        break;
                    }
                }
            }
            */
        });

        let logic_thread = logic_builder.spawn(move || {
            //let logic = Logic::new(&config);
            //logic.run(net_receiver, net_syncsender);
        });

        if thread::panicking() {
            println!("dropped while unwinding");
        } else {
            println!("dropped while not unwinding");
        }

        loop {
            if self.is_exit {
                break;
            } else {
                thread::sleep(Duration::from_secs(1))
            }
        }
    }

    pub fn exit(&mut self) {
        self.is_exit = true;
    }
}
