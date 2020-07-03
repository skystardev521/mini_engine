use log::error;
use socket::message::NetMsg;
use socket::tcp_server::TcpServer;
use socket::tcp_server_config::TcpServerConfig;
use socket::tcp_server_config::TcpServerConfigBuilder;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread;
use std::time::Duration;

mod logic_config;
mod logic_server;

fn main() {
    println!("Hello, world!");

    let tcp_server_confg: TcpServerConfig;
    match read_tcp_server_config("tcp_server_confg.txt".into()) {
        Ok(config) => tcp_server_confg = config,
        Err(err) => {
            error!("read_tcp_server_config error:{}", err);
            return;
        }
    }

    let logic_config: logic_config::LogicConfig;
    match read_logic_server_config("logic_config.txt".into()) {
        Ok(config) => logic_config = config,
        Err(err) => {
            error!("LogicConfig error:{}", err);
            return;
        }
    }

    let channel_size = 10000;

    let net_builder = thread::Builder::new().name("Net".into()); //.stack_size(stack_size);
    let logic_builder = thread::Builder::new().name("Logic".into());

    //阻塞模式 设置队列大小
    let (net_sync_sender, logic_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
        mpsc::sync_channel(channel_size);
    let (logic_sync_sender, net_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
        mpsc::sync_channel(channel_size);

    let _net_thread = net_builder.spawn(move || {
        net_thread_run(&tcp_server_confg, net_receiver, net_sync_sender);
    });

    let _logic_thread = logic_builder.spawn(move || {
        logic_thread(&logic_config, logic_receiver, logic_sync_sender);
    });

    loop {
        thread::sleep(Duration::from_secs(60))
    }
}

fn read_tcp_server_config(_path: String) -> Result<TcpServerConfig, String> {
    let config_builder = TcpServerConfigBuilder::new();
    let config = config_builder.builder();
    Ok(config)
}

fn read_logic_server_config(_path: String) -> Result<logic_config::LogicConfig, String> {
    let config_builder = logic_config::LogicConfigBuilder::new();
    let config = config_builder.builder();
    Ok(config)
}

fn net_thread_run(
    config: &TcpServerConfig,
    receiver: Receiver<NetMsg>,
    sync_sender: SyncSender<NetMsg>,
) {
    let mut new_net_msg_cb = |net_msg| match sync_sender.try_send(net_msg) {
        Ok(()) => (), //return true,
        Err(TrySendError::Full(_)) => {
            error!("net sync_sender Full");
        }
        Err(TrySendError::Disconnected(_)) => {
            error!("net sync_sender Disconnected");
        }
    };

    let mut tcp_server: TcpServer;
    match TcpServer::new(&config, &mut new_net_msg_cb) {
        Ok(server) => tcp_server = server,
        Err(err) => {
            error!("TcpServer::new error:{}", err);
            return;
        }
    }

    loop {
        if let Err(err) = tcp_server.wait_socket_event() {
            error!("error:{}", err);
            break;
        }
        loop {
            match receiver.try_recv() {
                Ok(net_msg) => {
                    tcp_server.write_net_msg(net_msg);
                }
                Err(TryRecvError::Empty) => break,

                Err(TryRecvError::Disconnected) => {
                    error!("net receiver Disconnected");
                    break;
                }
            }
        }
    }
}

fn logic_thread(
    config: &logic_config::LogicConfig,
    receiver: Receiver<NetMsg>,
    sync_sender: SyncSender<NetMsg>,
) {
    let timeout = Duration::from_millis(1);

    let mut new_net_msg_cb = |net_msg| match sync_sender.try_send(net_msg) {
        Ok(()) => (), //return true,
        Err(TrySendError::Full(_)) => {
            error!("net sync_sender Full");
        }
        Err(TrySendError::Disconnected(_)) => {
            error!("net sync_sender Disconnected");
        }
    };
    let mut logic_server = logic_server::LogicServer::new(&config, &mut new_net_msg_cb);
    loop {
        loop {
            match receiver.recv_timeout(timeout) {
                Ok(net_msg) => {
                    logic_server.new_net_msg(net_msg);
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }
}
