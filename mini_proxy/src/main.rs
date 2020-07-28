use log::error;
use mini_socket::message::NetMsg;
use mini_socket::message::ProtoId;
use mini_socket::tcp_listen_config::TcpListenConfig;
use mini_socket::tcp_listen_service::TcpListenService;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread;
use std::time::Duration;

use mini_utils::logger::Logger;

use crate::wan_service::WanService;
use config::ProxyConfig;
use config::WorkerConfig;

mod config;
mod wan_service;
mod worker;

use mini_utils::time;

const LOG_FILE_DURATION: u64 = 60 * 60 * 1000;

fn main() {
    let mut open_log_file_ts = time::timestamp();
    match Logger::init(&String::from("info"), &String::from("logs/mini_proxy.log")) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    let tcp_server_confg: TcpListenConfig;
    match read_tcp_server_config("tcp_server_confg.txt".into()) {
        Ok(config) => tcp_server_confg = config,
        Err(err) => {
            error!("read_tcp_server_config error:{}", err);
            return;
        }
    }

    let config: ProxyConfig;
    match read_proxy_config("config.txt".into()) {
        Ok(cfg) => config = cfg,
        Err(err) => {
            error!("Config error:{}", err);
            return;
        }
    }

    /*
    let channel_size = 10000;

    let net_builder = thread::Builder::new().name("Net".into()); //.stack_size(stack_size);
    let logic_builder = thread::Builder::new().name("Logic".into());

    //阻塞模式 设置队列大小
    let (net_sync_sender, agent_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
        mpsc::sync_channel(channel_size);
    let (agent_sync_sender, net_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
        mpsc::sync_channel(channel_size);

    let _net_thread = net_builder.spawn(move || {
        net_thread_run(&tcp_server_confg, net_receiver, net_sync_sender);
    });

    let _agent_thread = logic_builder.spawn(move || {
        agent_thread_run(&config, agent_receiver, agent_sync_sender);
    });

    */
    loop {
        thread::sleep(Duration::from_secs(60));

        if open_log_file_ts + LOG_FILE_DURATION < time::timestamp() {
            log::logger().flush();
            open_log_file_ts = time::timestamp();
        }
    }
}

fn read_tcp_server_config(_path: String) -> Result<TcpListenConfig, String> {
    Ok(TcpListenConfig::new())
}

fn read_proxy_config(_path: String) -> Result<ProxyConfig, String> {
    Ok(ProxyConfig::new(String::from("proxy"), WorkerConfig::new()))
}

/*
fn net_thread_run(
    config: &TcpListenConfig,
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

    let mut tcp_listen_service: TcpListenService;
    match TcpListenService::new(&config, &mut net_msg_cb) {
        Ok(server) => tcp_listen_service = server,
        Err(err) => {
            error!("TcpListenService::new error:{}", err);
            return;
        }
    }

    let mut single_write_msg_count;
    let mut epoll_wait_timeout = 0;

    loop {
        tcp_listen_service.tick();

        match tcp_listen_service.epoll_event(epoll_wait_timeout) {
            Ok(0) => epoll_wait_timeout = 1,
            Ok(num) => {
                if num == config.epoll_max_events {
                    epoll_wait_timeout = 0;
                }
            }
            Err(err) => {
                error!("TcpListenService epoll_event:{}", err);
                break;
            }
        }

        loop {
            single_write_msg_count = 0;

            match receiver.try_recv() {
                Ok(net_msg) => {
                    //这里要优化 判断是否广播消息，广告
                    tcp_listen_service.write_net_msg(net_msg);

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

fn agent_thread_run(
    config: &ProxyConfig,
    receiver: Receiver<NetMsg>,
    sync_sender: SyncSender<NetMsg>,
) {
    let mut net_msg_cb = |net_msg: NetMsg| match sync_sender.try_send(net_msg) {
        Ok(()) => return Ok(()),
        Err(TrySendError::Full(_)) => {
            error!("agent_thread_run try_send Full");
            return Err(ProtoId::BusyServer);
        }
        Err(TrySendError::Disconnected(_)) => {
            error!("agent_thread_run try_send Disconnected");
            return Err(ProtoId::ExceptionServer);
        }
    };
    let mut server = Service::new(&config, &mut net_msg_cb);

    let mut single_task_num;
    let timeout_duration = Duration::from_millis(1);
    let single_max_task_num = config.worker_config.get_single_max_task_num();

    'next_loop: loop {
        server.tick();
        single_task_num = 0;
        match receiver.recv_timeout(timeout_duration) {
            Ok(net_msg) => {
                server.new_net_msg(net_msg);
                single_task_num += 1;
                if single_task_num == single_max_task_num {
                    continue 'next_loop;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                continue 'next_loop;
            }
            Err(RecvTimeoutError::Disconnected) => {
                error!("agent_thread_run recv_timeout Disconnected");
            }
        }
    }
}
*/
