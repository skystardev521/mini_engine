use crate::config::NetConfig;
//use crate::logic::Logic;
//use crate::net::Net;
use log::error;
use socket::clients::Clients;
use socket::epoll::Epoll;
use socket::message::NetMsg;
use socket::tcp_event::TcpEvent;
use socket::tcp_listen::TcpListen;
use std::cell::RefCell;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread;
use std::time::Duration;

pub struct Engine {
    is_exit: bool,
}

impl Engine {
    /*
    pub fn init(config: &Config) -> Result<Self, String> {
        return OK(Engine { config: confg });
    }
    */

    pub fn new() -> Self {
        Engine { is_exit: false }
    }

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
        let (net_syncsender, _logic_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
            mpsc::sync_channel(channel_size);
        let (_logic_syncsender, net_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
            mpsc::sync_channel(channel_size);

        let _net_thread = net_builder.spawn(move || {
            if let Err(err) = net_handle(&net_cfg, net_receiver, net_syncsender) {
                error!("net thread quit result:{}", err)
            }
        });

        let _logic_thread = logic_builder.spawn(move || {
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
                thread::sleep(Duration::from_secs(1000))
            }
        }
    }

    pub fn exit(&mut self) {
        self.is_exit = true;
    }
}

fn net_handle(
    cfg: &NetConfig,
    receiver: Receiver<NetMsg>,
    sync_sender: SyncSender<NetMsg>,
) -> Result<(), String> {
    let epoll: Epoll = Epoll::new()?;

    //let clients = RefCell::new(Clients::new(cfg.max_client, cfg.msg_max_size, &epoll)?);

    let mut clients = Clients::new(cfg.max_client, cfg.msg_max_size, &epoll)?;

    let mut is_exit_thread = false;
    let mut msg_data_cb = |net_msg: NetMsg| match sync_sender.try_send(net_msg) {
        Ok(()) => return true,
        Err(TrySendError::Full(_)) => return false,
        Err(TrySendError::Disconnected(_)) => {
            is_exit_thread = true;
            error!("net sync_sender Disconnected");
            return false;
        }
    };

    let mut tcp_event = TcpEvent::new(&epoll, &mut clients, &mut msg_data_cb);

    //let mut tcp_event = TcpEvent::new(&epoll, &clients, &mut msg_data_cb);

    let mut tcp_listen = TcpListen::new(
        &cfg.tcp_linsten_addr,
        cfg.epoll_max_events,
        &epoll,
        &mut tcp_event,
    )?;

    let mut is_exit_loop = false;
    loop {
        if is_exit_loop {
            return Err("exit loop".into());
        }

        match tcp_listen.wait(cfg.epoll_wait_timeout) {
            Ok(true) => (),
            Ok(false) => (),
            Err(err) => {
                error!("tcp_listen.run:{}", err);
                is_exit_loop = true;
            }
        }
        loop {
            match receiver.try_recv() {
                Ok(net_msg) => {
                    write_msg_data(&mut clients, net_msg);
                }
                Err(TryRecvError::Empty) => break,

                Err(TryRecvError::Disconnected) => {
                    error!("net receiver Disconnected");
                    is_exit_loop = true;
                    break;
                }
            }
        }
    }
}

fn write_msg_data(clients: &mut Clients, net_msg: NetMsg) {
    if let Some(client) = clients.get_client(net_msg.id) {
        if let Err(err) = client.tcp_writer.add_msg_data(net_msg.data) {
            error!("tcp_listen.run:{}", err);
        }
    }
}
