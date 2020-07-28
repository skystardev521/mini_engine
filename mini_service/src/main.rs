use log::error;
use mini_socket::message::NetMsg;
use mini_socket::message::ProtoId;
use mini_socket::tcp_connect_config::TcpConnectConfig;
use mini_socket::tcp_connect_config::TcpConnectConfigBuilder;
use mini_socket::tcp_connect_service::TcpConnectService;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread;
use std::time::Duration;

use mini_utils::logger::Logger;

use mini_service::Config;
use mini_service::ConfigBuilder;
use mini_service::Service;

use mini_utils::time;

const LOG_FILE_DURATION: u64 = 60 * 60 * 1000;

fn main() {
    let mut open_log_file_ts = time::timestamp();
    match Logger::init(
        &String::from("info"),
        &String::from("logs/mini_service.log"),
    ) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    let tcp_connect_confg: TcpConnectConfig;
    match read_tcp_connect_confg("tcp_connect_confg.txt".into()) {
        Ok(config) => tcp_connect_confg = config,
        Err(err) => {
            error!("read_tcp_connect_confg error:{}", err);
            return;
        }
    }

    let config: Config;
    match read_server_config("config.txt".into()) {
        Ok(cfg) => config = cfg,
        Err(err) => {
            error!("Config error:{}", err);
            return;
        }
    }

    let channel_size = 10000;

    let net_builder = thread::Builder::new().name("Net".into()); //.stack_size(stack_size);
    let server_builder = thread::Builder::new().name("Logic".into());

    //阻塞模式 设置队列大小
    let (net_sync_sender, server_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
        mpsc::sync_channel(channel_size);
    let (server_sync_sender, net_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
        mpsc::sync_channel(channel_size);

    let _net_thread = net_builder.spawn(move || {
        net_thread_run(&tcp_connect_confg, net_receiver, net_sync_sender);
    });

    let _server_thread = server_builder.spawn(move || {
        server_thread_run(&config, server_receiver, server_sync_sender);
    });

    loop {
        thread::sleep(Duration::from_secs(60));

        if open_log_file_ts + LOG_FILE_DURATION < time::timestamp() {
            log::logger().flush();
            open_log_file_ts = time::timestamp();
        }
    }
}

fn read_tcp_connect_confg(_path: String) -> Result<TcpConnectConfig, String> {
    let mut config_builder = TcpConnectConfigBuilder::new();
    let config = config_builder
        .set_vec_socket_addr(&vec!["0.0.0.0:9999".into()])
        .builder();
    Ok(config)
}

fn read_server_config(_path: String) -> Result<Config, String> {
    let config_builder = ConfigBuilder::new();
    let config = config_builder.builder();
    Ok(config)
}

fn net_thread_run(
    config: &TcpConnectConfig,
    receiver: Receiver<NetMsg>,
    sync_sender: SyncSender<NetMsg>,
) {
    let mut net_msg_cb = |net_msg: NetMsg| {
        match sync_sender.try_send(net_msg) {
            Ok(()) => return Ok(()),
            Err(TrySendError::Full(_)) => {
                error!("net_thread try_send Full");
                return Err(ProtoId::BusyServer);
            }
            Err(TrySendError::Disconnected(_)) => {
                error!("net_thread try_send Disconnected");
                return Err(ProtoId::ExceptionServer);
            }
        };
    };

    let mut tcp_connect_service: TcpConnectService;
    match TcpConnectService::new(&config, &mut net_msg_cb) {
        Ok(server) => tcp_connect_service = server,
        Err(err) => {
            error!("TcpConnectService::new error:{}", err);
            return;
        }
    }

    let mut single_write_msg_count;
    let mut epoll_wait_timeout = 0;

    loop {
        tcp_connect_service.tick();

        match tcp_connect_service.epoll_event(epoll_wait_timeout) {
            Ok(0) => epoll_wait_timeout = 1,
            Ok(num) => {
                if num == config.epoll_max_events {
                    epoll_wait_timeout = 0;
                }
            }
            Err(err) => {
                error!("TcpConnectService epoll_event:{}", err);
                break;
            }
        }

        loop {
            single_write_msg_count = 0;

            match receiver.try_recv() {
                Ok(net_msg) => {
                    //这里要优化 判断是否广播消息，广告
                    tcp_connect_service.write_net_msg(net_msg);

                    single_write_msg_count += 1;
                    if single_write_msg_count == config.single_write_msg_max_num {
                        epoll_wait_timeout = 0;
                        break;
                    }
                }
                Err(TryRecvError::Empty) => break,

                Err(TryRecvError::Disconnected) => {
                    error!("net_thread receiver.try_recv:Disconnected");
                    break;
                }
            }
        }
    }
}

fn server_thread_run(config: &Config, receiver: Receiver<NetMsg>, sync_sender: SyncSender<NetMsg>) {
    let mut net_msg_cb = |net_msg: NetMsg| match sync_sender.try_send(net_msg) {
        Ok(()) => return Ok(()),
        Err(TrySendError::Full(_)) => {
            error!("net_thread try_send Full");
            return Err(ProtoId::BusyServer);
        }
        Err(TrySendError::Disconnected(_)) => {
            error!("net_thread try_send Disconnected");
            return Err(ProtoId::ExceptionServer);
        }
    };
    let mut server = Service::new(&config, &mut net_msg_cb);

    let mut single_read_msg_count;

    let timeout_duration = Duration::from_millis(1);

    'next_loop: loop {
        server.tick();
        single_read_msg_count = 0;
        match receiver.recv_timeout(timeout_duration) {
            Ok(net_msg) => {
                server.new_net_msg(net_msg);
                single_read_msg_count += 1;
                if single_read_msg_count == config.single_read_msg_max_num {
                    continue 'next_loop;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                continue 'next_loop;
            }
            Err(RecvTimeoutError::Disconnected) => {
                error!("agent_thread recv_timeout Disconnected");
            }
        }
    }
}
