use log::error;
use socket::message::NetMsg;
use socket::tcp_server::TcpServer;
use socket::tcp_server_config;
use socket::tcp_server_config::TcpServerConfigBuilder;
use socket::tcp_socket::TcpSocket;
use socket::tcp_socket_mgmt::TcpSocketMgmt;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Hello, world!");

    let config_builder = TcpServerConfigBuilder::new();
    let tcp_server_confg = config_builder.builder();
    let channel_size = 10000;

    let net_builder = thread::Builder::new().name("Net".into()); //.stack_size(stack_size);
    let logic_builder = thread::Builder::new().name("Logic".into());

    //阻塞模式 设置队列大小
    let (net_syncsender, _logic_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
        mpsc::sync_channel(channel_size);
    let (_logic_syncsender, net_receiver): (SyncSender<NetMsg>, Receiver<NetMsg>) =
        mpsc::sync_channel(channel_size);

    let _net_thread = net_builder.spawn(move || {
        /*
        if let Err(err) = net_handle(&net_cfg, net_receiver, net_syncsender) {
            error!("net thread quit result:{}", err)
        }
        */

        let mut tcp_server: TcpServer;
        match TcpServer::new(&tcp_server_confg) {
            Ok(server) => tcp_server = server,
            Err(err) => {
                error!("TcpServer::new error:{}", err);
                return;
            }
        }

        loop {
            match tcp_server.epoll_wait() {
                Ok(msg_num) => (),
                Err(err) => error!("error:{}", err),
            }
            loop {
                match net_receiver.try_recv() {
                    Ok(net_msg) => {
                        tcp_server.write_msg_data(net_msg);
                    }
                    Err(TryRecvError::Empty) => break,

                    Err(TryRecvError::Disconnected) => {
                        error!("net receiver Disconnected");
                        //is_exit_loop = true;
                        break;
                    }
                }
            }
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

    /*
    loop {
        if self.is_exit {
            break;
        } else {
            thread::sleep(Duration::from_secs(1000))
        }
    }
    */
}

fn write_msg_data(tcp_socket_mgmt: &mut TcpSocketMgmt, net_msg: NetMsg) {
    if let Some(tcp_socket) = tcp_socket_mgmt.get_tcp_socket(net_msg.id) {
        if let Err(err) = tcp_socket.writer.add_msg_data(net_msg.data) {
            error!("tcp_listen.run:{}", err);
        }
    }
}
