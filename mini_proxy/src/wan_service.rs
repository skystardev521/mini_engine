use crate::net_message::WanMsgEnum;
use crate::wan_buf_rw::WanBufRw;
use mini_socket::msg_kind::MsgKind;
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
    worker: Worker<WanMsgEnum, ()>,
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
    pub fn receiver(&self) -> Option<WanMsgEnum> {
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
    pub fn sender(&self, msg: WanMsgEnum) -> bool {
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
) -> Box<dyn FnOnce(Receiver<WanMsgEnum>, SyncSender<WanMsgEnum>) + Send> {
    Box::new(
        move |receiver: Receiver<WanMsgEnum>, sender: SyncSender<WanMsgEnum>| {
            //-----------------------------------------------------------------------------
            let mut net_msg_cb_fn = |sid: u64, vec_msg: Vec<Vec<u8>>| {
                for msg in vec_msg {
                    match sender.try_send(WanMsgEnum::NetMsg(sid, msg)) {
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
            let mut msg_kind_cb_fn = |sid: u64, msg: MsgKind| {
                match sender.try_send(WanMsgEnum::MsgKind(sid, msg)) {
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
            let mut tcp_listen_service: TcpListenService<WanBufRw, Vec<u8>>;
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
                        Ok(WanMsgEnum::NetMsg(sid, msg)) => {
                            //这里要优化 判断是否广播消息
                            tcp_listen_service.write_net_msg(sid, msg);
                        }
                        Ok(WanMsgEnum::MsgKind(sid, msg)) => {
                            if msg == MsgKind::CloseSocket {
                                tcp_listen_service.del_tcp_socket(sid);
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
