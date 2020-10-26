use crate::head_proto::wan::{MsgEnum, NetMsg};

use crate::wan_tcp_rw::WanTcpRw;
use mini_socket::exc_kind::ExcKind;
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
            RecvResEnum::Data(msg_enum) => {
                return Some(msg_enum);
            }
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
                error!("Worker:{} Sender Full", self.worker.get_name());
                return false;
            }
            SendResEnum::Disconnected(_) => {
                error!("Worker:{} Disconnected", self.worker.get_name());
                return false;
            }
        }
    }
}

#[allow(dead_code)]
fn worker_closure(
    tcp_listen_config: TcpListenConfig,
) -> Box<dyn FnOnce(Receiver<MsgEnum>, SyncSender<MsgEnum>) + Send> {
    Box::new(
        move |receiver: Receiver<MsgEnum>, sender: SyncSender<MsgEnum>| {
            //-----------------------------------------------------------------------------
            let mut net_msg_cb_fn = |cid: u32, vec_msg: Vec<NetMsg>| {
                for msg in vec_msg {
                    match sender.try_send(MsgEnum::NetMsg(cid, msg)) {
                        Ok(_) => {}
                        Err(TrySendError::Full(_)) => {
                            error!("WanService try_send Full");
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            error!("WanService try_send Disconnected");
                        }
                    };
                }
            };
            let mut msg_kind_cb_fn = |cid: u32, ekd: ExcKind| {
                match sender.try_send(MsgEnum::ExcMsg(cid, ekd)) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {
                        error!("WanService try_send Full");
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        error!("WanService try_send Disconnected");
                    }
                };
            };
            //-----------------------------------------------------------------------------
            let mut tcp_listen_service: TcpListenService<WanTcpRw, NetMsg>;
            match TcpListenService::new(&tcp_listen_config, &mut net_msg_cb_fn, &mut msg_kind_cb_fn)
            {
                Ok(service) => tcp_listen_service = service,
                Err(err) => {
                    error!("TcpListenService::new error:{}", err);
                    return;
                }
            }
            //-----------------------------------------------------------------------------

            let wait_timeout = tcp_listen_config.epoll_wait_timeout;

            loop {
                tcp_listen_service.tick();
                loop {
                    match tcp_listen_service.epoll_event(wait_timeout) {
                        Ok(0) => {
                            break;
                        }
                        Ok(_) => {}
                        Err(err) => {
                            error!("TcpListenService epoll_event:{}", err);
                            break;
                        }
                    }
                }
                //-----------------------------------------------------------------------------
                //single_write_msg_count = 0;
                loop {
                    match receiver.try_recv() {
                        Ok(MsgEnum::NetMsg(cid, msg)) => {
                            //这里要优化 判断是否广播消息
                            tcp_listen_service.write_net_msg(cid, msg);
                        }
                        Ok(MsgEnum::ExcMsg(cid, ekd)) => {
                            if ekd == ExcKind::CloseSocket {
                                tcp_listen_service.del_tcp_socket(cid);
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
