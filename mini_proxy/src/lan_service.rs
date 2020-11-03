use crate::lan_tcp_rw::LanTcpRw;
use mini_socket::tcp_socket_msg::{SrvMsg,MsgData,SProtoId};
use mini_socket::tcp_listen_config::TcpListenConfig;
use mini_socket::tcp_listen_service::TcpListenService;
use mini_utils::wconfig::WConfig;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;

use log::error;
use mini_utils::worker::RecvResEnum;
use mini_utils::worker::SendResEnum;
use mini_utils::worker::Worker;

/// 收发广域网的数据
pub struct LanService {
    worker: Worker<SrvMsg, ()>,
}

impl LanService {
    pub fn new(
        workers_config: &WConfig,
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
    pub fn receiver(&self) -> Option<SrvMsg> {
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
    pub fn sender(&self, msg: SrvMsg) -> bool {
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
) -> Box<dyn FnOnce(Receiver<SrvMsg>, SyncSender<SrvMsg>) + Send> {
    Box::new(
        move |receiver: Receiver<SrvMsg>, sender: SyncSender<SrvMsg>| {
            //-----------------------------------------------------------------------------
            let mut net_msg_cb_fn = |sid: u64, vec_msg: Vec<MsgData>| {
                for msg in vec_msg {
                    match sender.try_send(SrvMsg::new(sid, msg)) {
                        Ok(_) => {}
                        Err(TrySendError::Full(_)) => {
                            error!("LanService try_send Full");
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            error!("LanService try_send Disconnected");
                        }
                    };
                }
            };
            let mut msg_kind_cb_fn = |sid: u64, spid: SProtoId| {
                match sender.try_send(SrvMsg::new(sid, MsgData::new_pid(spid as u16))) {
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
            let mut tcp_listen_service: TcpListenService<LanTcpRw, MsgData>;
            match TcpListenService::new(&tcp_listen_config, &mut net_msg_cb_fn, &mut msg_kind_cb_fn)
            {
                Ok(service) => {
                    tcp_listen_service = service;
                }
                Err(err) => {
                    error!("TcpListenService::new error:{}", err);
                    return;
                }
            }

            //-----------------------------------------------------------------------------
            let wait_timeout = 1;
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
                        Ok(srv_msg) => {
                            //这里要优化 判断是否广播消息
                            tcp_listen_service.write_msg(srv_msg.id, srv_msg.msg);
                        }
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
