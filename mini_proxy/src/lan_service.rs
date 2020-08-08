use crate::lan_buf_rw::LanBufRw;
use crate::net_message::LanErrMsg;
use crate::net_message::LanMsgEnum;
use crate::net_message::LanNetMsg;
use mini_socket::message::ErrMsg;
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
pub struct LanService {
    worker: Worker<LanMsgEnum, ()>,
}

impl LanService {
    pub fn new(
        workers_config: &WorkerConfig,
        tcp_listen_config: TcpListenConfig,
    ) -> Result<Self, String> {
        let worker = Worker::new(
            String::from("LanService"),
            workers_config.get_stack_size(),
            workers_config.get_channel_size(),
            worker_closure(tcp_listen_config),
        )?;

        Ok(LanService { worker: worker })
    }

    #[inline]
    pub fn receiver(&self) -> Option<LanMsgEnum> {
        match self.worker.receiver() {
            RecvResEnum::Empty => return None,
            RecvResEnum::Data(lan_msg) => return Some(lan_msg),
            RecvResEnum::Disconnected => {
                error!("Worker:{} Disconnected", self.worker.get_name());
                return None;
            }
        }
    }
    #[inline]
    pub fn sender(&self, msg: LanMsgEnum) -> bool {
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

fn worker_closure(
    tcp_listen_config: TcpListenConfig,
) -> Box<dyn FnOnce(Receiver<LanMsgEnum>, SyncSender<LanMsgEnum>) + Send> {
    Box::new(
        move |receiver: Receiver<LanMsgEnum>, sender: SyncSender<LanMsgEnum>| {
            //-----------------------------------------------------------------------------
            let mut net_msg_cb_fn = |sid: u64, net_msg: LanNetMsg| {
                match sender.try_send(LanMsgEnum::NetMsg(sid, net_msg)) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {
                        error!("LanService try_send Full");
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        error!("LanService try_send Disconnected");
                    }
                };
            };
            let mut err_msg_cb_fn = |sid: u64, err_msg: ErrMsg| {
                match sender.try_send(LanMsgEnum::ErrMsg(
                    sid,
                    LanErrMsg {
                        sid: sid,
                        data: err_msg,
                    },
                )) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {
                        error!("LanService try_send Full");
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        error!("LanService try_send Disconnected");
                    }
                };
            };
            //-----------------------------------------------------------------------------
            let mut tcp_listen_service: TcpListenService<LanBufRw, LanNetMsg>;
            match TcpListenService::new(&tcp_listen_config, &mut net_msg_cb_fn, &mut err_msg_cb_fn)
            {
                Ok(service) => tcp_listen_service = service,
                Err(err) => {
                    error!("TcpListenService::new error:{}", err);
                    return;
                }
            }

            //-----------------------------------------------------------------------------
            let wait_timeout = 1;
            //let mut single_write_msg_count;
            let mut single_call_epoll_wait_count;
            let single_write_msg_max_num = tcp_listen_config.single_write_msg_max_num;
            let single_call_epoll_wait_max_num = tcp_listen_config.single_call_epoll_wait_max_num;
            loop {
                tcp_listen_service.tick();
                single_call_epoll_wait_count = 0;
                loop {
                    match tcp_listen_service.epoll_event(wait_timeout) {
                        Ok(0) => {
                            break;
                        }
                        Ok(_) => {
                            /*
                            single_call_epoll_wait_count += 1;
                            if single_call_epoll_wait_count == single_call_epoll_wait_max_num {
                                break;
                            }
                            */
                        }
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
                        Ok(LanMsgEnum::NetMsg(sid, net_msg)) => {
                            //这里要优化 判断是否广播消息
                            tcp_listen_service.write_net_msg(sid, net_msg);
                            /*
                            single_write_msg_count += 1;
                            if single_write_msg_count == single_write_msg_max_num {
                                break;
                            }
                            */
                        }
                        Ok(LanMsgEnum::ErrMsg(_sid, _err_msg)) => {}
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            error!("LanService receiver.try_recv:Disconnected");
                            break;
                        }
                    }
                }
                //-----------------------------------------------------------------------------
            }
        },
    )
}
