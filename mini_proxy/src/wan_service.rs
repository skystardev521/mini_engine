use mini_socket::message::MsgEnum;
use mini_socket::message::SysMsgId;
use mini_socket::tcp_listen_config::TcpListenConfig;
use mini_socket::tcp_listen_service::TcpListenService;
use mini_utils::worker_config::WorkerConfig;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;

use log::{error, warn};
use mini_utils::worker::RecvResEnum;
use mini_utils::worker::SendResEnum;
use mini_utils::worker::Worker;

/// 收发广域网的数据
pub struct WanService {
    worker: Worker<MsgEnum, ()>,
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

        Ok(WanService { worker: worker })
    }

    #[inline]
    pub fn receiver(&self) -> Option<MsgEnum> {
        match self.worker.receiver() {
            RecvResEnum::Empty => return None,
            RecvResEnum::Data(net_msg) => return Some(net_msg),
            RecvResEnum::Disconnected => {
                error!("Worker:{} Disconnected", self.worker.get_name());
                return None;
            }
        }
    }
    #[inline]
    pub fn sender(&self, msg: MsgEnum) -> bool {
        match self.worker.sender(msg) {
            SendResEnum::Success => {
                return true;
            }
            SendResEnum::Full(_) => {
                warn!("Worker:{} Full", self.worker.get_name());
                return false;
            }
            SendResEnum::Disconnected(_) => {
                error!("Worker:{} Disconnected", self.worker.get_name());
                return false;
            }
        }
    }
}

fn worker_closure(
    tcp_listen_config: TcpListenConfig,
) -> Box<dyn FnOnce(Receiver<MsgEnum>, SyncSender<MsgEnum>) + Send> {
    Box::new(
        move |receiver: Receiver<MsgEnum>, sender: SyncSender<MsgEnum>| {
            //-----------------------------------------------------------------------------
            let mut msg_cb = |msg: MsgEnum| {
                match sender.try_send(msg) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {
                        error!("WanService try_send Full");
                        //return Err(ProtoId::BusyServer);
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        error!("WanService try_send Disconnected");
                        //return Err(ProtoId::ExceptionServer);
                    }
                };
            };
            //-----------------------------------------------------------------------------
            let mut tcp_listen_service: TcpListenService;
            match TcpListenService::new(&tcp_listen_config, &mut msg_cb) {
                Ok(service) => tcp_listen_service = service,
                Err(err) => {
                    error!("TcpListenService::new error:{}", err);
                    return;
                }
            }
            //-----------------------------------------------------------------------------
            let mut epoll_wait_timeout = 0;
            let mut single_write_msg_count;
            let single_write_msg_max_num = tcp_listen_config.single_write_msg_max_num;
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
                        Ok(MsgEnum::NetMsg(net_msg)) => {
                            //这里要优化 判断是否广播消息
                            tcp_listen_service.write_net_msg(net_msg);
                            single_write_msg_count += 1;
                            if single_write_msg_count == single_write_msg_max_num {
                                epoll_wait_timeout = 0;
                                break;
                            }
                        }
                        Ok(MsgEnum::SysMsg(sys_msg)) => {
                            if sys_msg.smid == SysMsgId::CloseSocket {
                                tcp_listen_service.del_tcp_socket(sys_msg.sid, false);
                            }
                        }
                        Err(TryRecvError::Empty) => break,

                        Err(TryRecvError::Disconnected) => {
                            error!("WanService receiver.try_recv:Disconnected");
                            break;
                        }
                    }
                }
                //-----------------------------------------------------------------------------
            }
        },
    )
}
