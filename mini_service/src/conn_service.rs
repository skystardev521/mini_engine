use crate::net_buf_rw::NetBufRw;
use crate::net_message::MsgEnum;
use crate::net_message::NetMsg;
use mini_socket::msg_kind::MsgKind;
use mini_socket::tcp_connect_config::TcpConnectConfig;
use mini_socket::tcp_connect_service::TcpConnectService;
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
pub struct ConnService {
    worker: Worker<MsgEnum, ()>,
}

impl ConnService {
    pub fn new(
        workers_config: &WorkerConfig,
        vec_tcp_connect_config: Vec<TcpConnectConfig>,
    ) -> Result<Self, String> {
        //let max_task_num = workers_config.get_single_max_task_num();
        let worker = Worker::new(
            String::from("ConnService"),
            workers_config.get_stack_size(),
            workers_config.get_channel_size(),
            worker_closure(vec_tcp_connect_config),
        )?;

        Ok(ConnService { worker: worker })
    }

    #[inline]
    pub fn receiver(&self) -> Option<MsgEnum> {
        match self.worker.receiver() {
            RecvResEnum::Empty => return None,
            RecvResEnum::Data(msg) => return Some(msg),
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
    vec_tcp_connect_config: Vec<TcpConnectConfig>,
) -> Box<dyn FnOnce(Receiver<MsgEnum>, SyncSender<MsgEnum>) + Send> {
    Box::new(
        move |receiver: Receiver<MsgEnum>, sender: SyncSender<MsgEnum>| {
            //-----------------------------------------------------------------------------
            let mut net_msg_cb_fn = |sid: u64, vec_msg: Vec<NetMsg>| {
                for msg in vec_msg {
                    match sender.try_send(MsgEnum::NetMsg(sid, msg)) {
                        Ok(_) => {}
                        Err(TrySendError::Full(_)) => {
                            error!("TcpConnectService try_send Full");
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            error!("TcpConnectService try_send Disconnected");
                        }
                    };
                }
            };

            let mut msg_kind_cb_fn = |sid: u64, err_msg: MsgKind| {
                match sender.try_send(MsgEnum::MsgKind(sid, err_msg)) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {
                        error!("TcpConnectService try_send Full");
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        error!("TcpConnectService try_send Disconnected");
                    }
                };
            };
            //-----------------------------------------------------------------------------
            let mut tcp_connect_service: TcpConnectService<NetBufRw, NetMsg>;
            match TcpConnectService::new(
                vec_tcp_connect_config,
                &mut net_msg_cb_fn,
                &mut msg_kind_cb_fn,
            ) {
                Ok(service) => tcp_connect_service = service,
                Err(err) => {
                    error!("TcpConnectService::new error:{}", err);
                    return;
                }
            }
            //-----------------------------------------------------------------------------
            let wait_timeout = 1;
            loop {
                tcp_connect_service.tick();
                loop {
                    match tcp_connect_service.epoll_event(wait_timeout) {
                        Ok(0) => {
                            break;
                        }
                        Ok(_) => {}
                        Err(err) => {
                            error!("tcp_connect_service epoll_event:{}", err);
                            break;
                        }
                    }
                }
                //-----------------------------------------------------------------------------
                loop {
                    match receiver.try_recv() {
                        Ok(MsgEnum::NetMsg(sid, net_msg)) => {
                            //这里要优化 判断是否广播消息
                            tcp_connect_service.write_net_msg(sid, net_msg);
                        }
                        Ok(MsgEnum::MsgKind(_sid, _sys_msg)) => {}
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            error!("ConnService receiver.try_recv:Disconnected");
                            break;
                        }
                    }
                }
                //-----------------------------------------------------------------------------
            }
        },
    )
}
