use log::error;
use socket::message::NetMsg;
use socket::message::ProtoId;
use socket::tcp_listen_server::TcpListenServer;
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

use utils::logger::Logger;

use net_agent::Config;
use net_agent::ConfigBuilder;
use net_agent::Server;

use utils::time;

const LOG_FILE_DURATION: u64 = 60 * 60 * 1000;

fn main() {
    let mut open_log_file_ts = time::timestamp();
    match Logger::init(&String::from("info"), &String::from("logs/net_agent.log")) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    let tcp_server_confg: TcpServerConfig;
    match read_tcp_server_config("tcp_server_confg.txt".into()) {
        Ok(config) => tcp_server_confg = config,
        Err(err) => {
            error!("read_tcp_server_config error:{}", err);
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

    loop {
        thread::sleep(Duration::from_secs(60));

        if open_log_file_ts + LOG_FILE_DURATION < time::timestamp() {
            log::logger().flush();
            open_log_file_ts = time::timestamp();
        }
    }
}

fn read_tcp_server_config(_path: String) -> Result<TcpServerConfig, String> {
    let config_builder = TcpServerConfigBuilder::new();
    let config = config_builder.builder();
    Ok(config)
}

fn read_server_config(_path: String) -> Result<Config, String> {
    let config_builder = ConfigBuilder::new();
    let config = config_builder.builder();
    Ok(config)
}

fn net_thread_run(
    config: &TcpServerConfig,
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

    let mut tcp_listen_server: TcpListenServer;
    match TcpListenServer::new(&config, &mut net_msg_cb) {
        Ok(server) => tcp_listen_server = server,
        Err(err) => {
            error!("TcpListenServer::new error:{}", err);
            return;
        }
    }

    let mut single_write_msg_count;
    let mut epoll_wait_timeout = 0;

    loop {
        tcp_listen_server.tick();

        match tcp_listen_server.epoll_event(epoll_wait_timeout) {
            Ok(0) => epoll_wait_timeout = 1,
            Ok(num) => {
                if num == config.epoll_max_events {
                    epoll_wait_timeout = 0;
                }
            }
            Err(err) => {
                error!("TcpListenServer epoll_event:{}", err);
                break;
            }
        }

        loop {
            single_write_msg_count = 0;

            match receiver.try_recv() {
                Ok(net_msg) => {
                    //这里要优化 判断是否广播消息，广告
                    tcp_listen_server.write_net_msg(net_msg);

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

fn agent_thread_run(config: &Config, receiver: Receiver<NetMsg>, sync_sender: SyncSender<NetMsg>) {
    let mut net_msg_cb = |net_msg: NetMsg| {
        match sync_sender.try_send(net_msg) {
            Ok(()) => (), //return true,
            Err(TrySendError::Full(_)) => {
                //error!("agent_thread try_send Full");
            }
            Err(TrySendError::Disconnected(_)) => {
                error!("agent_thread try_send Disconnected");
            }
        }
    };
    let mut server = Server::new(&config, &mut net_msg_cb);

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
