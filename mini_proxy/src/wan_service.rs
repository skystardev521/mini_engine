use crate::config::WorkerConfig;
use mini_socket::message::NetMsg;
use mini_socket::message::ProtoId;
use mini_socket::tcp_listen_config::TcpListenConfig;
use mini_socket::tcp_listen_service::TcpListenService;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;

use crate::worker::Worker;

use log::error;

/// 收发广域网的数据
pub struct WanService {
    single_max_task_num: u16,
    worker: Worker<NetMsg, ()>,
}

impl WanService {
    pub fn new(
        workers_config: &WorkerConfig,
        tcp_listen_config: TcpListenConfig,
    ) -> Result<Self, String> {
        let worker = Worker::new(
            String::from("WanWorker"),
            workers_config.get_stack_size(),
            workers_config.get_channel_size(),
            worker_closure(tcp_listen_config),
        )?;

        Ok(WanService {
            worker: worker,
            single_max_task_num: workers_config.get_single_max_task_num(),
        })
    }

    #[inline]
    pub fn receiver(&mut self) {
        let mut num = 0;
        loop {
            match self.worker.receiver() {
                None => break,
                Some(msg) => {
                    num += 1;
                    if num == self.single_max_task_num {
                        break;
                    }
                }
            }
        }
    }
    #[inline]
    pub fn sender(&mut self, msg: NetMsg) -> bool {
        self.worker.sender(msg)
    }
}

/// service 逻辑
fn worker_closure(
    tcp_listen_config: TcpListenConfig,
) -> Box<dyn FnOnce(Receiver<NetMsg>, SyncSender<NetMsg>) + Send> {
    Box::new(
        move |receiver: Receiver<NetMsg>, sender: SyncSender<NetMsg>| {
            //-----------------------------------------------------------------------------
            let mut net_msg_cb = |net_msg: NetMsg| {
                match sender.try_send(net_msg) {
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
            //-----------------------------------------------------------------------------
            let mut tcp_listen_service: TcpListenService;
            match TcpListenService::new(&tcp_listen_config, &mut net_msg_cb) {
                Ok(server) => tcp_listen_service = server,
                Err(err) => {
                    error!("TcpListenService::new error:{}", err);
                    return;
                }
            }
            //-----------------------------------------------------------------------------
            let mut epoll_wait_timeout = 0;
            let mut single_write_msg_count;
            loop {
                tcp_listen_service.tick();

                match tcp_listen_service.epoll_event(epoll_wait_timeout) {
                    Ok(0) => epoll_wait_timeout = 1,
                    Ok(num) => {
                        if num == tcp_listen_config.epoll_max_events {
                            epoll_wait_timeout = 0;
                        }
                    }
                    Err(err) => {
                        error!("TcpListenService epoll_event:{}", err);
                        break;
                    }
                }
                //-----------------------------------------------------------------------------
                single_write_msg_count = 0;
                loop {
                    match receiver.try_recv() {
                        Ok(net_msg) => {
                            //这里要优化 判断是否广播消息
                            tcp_listen_service.write_net_msg(net_msg);

                            single_write_msg_count += 1;
                            if single_write_msg_count == tcp_listen_config.single_write_msg_max_num
                            {
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
                //-----------------------------------------------------------------------------
            }
        },
    )
}
