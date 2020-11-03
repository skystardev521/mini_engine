use mini_socket::tcp_socket_msg::{MsgData,SProtoId};

use crate::wan_tcp_rw::WanTcpRw;
use mini_socket::tcp_listen_config::TcpListenConfig;
use mini_socket::tcp_listen_service::TcpListenService;
use mini_utils::wconfig::WConfig;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;

use log::{error};
use mini_utils::worker::RecvResEnum;
use mini_utils::worker::SendResEnum;
use mini_utils::worker::Worker;

/// 收发广域网的数据
pub struct WanService {
    worker: Worker<MsgData, ()>,
}

impl WanService {
    pub fn new(
        workers_config: &WConfig,
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
    pub fn receiver(&self) -> Option<MsgData> {
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
    pub fn sender(&self, msg: MsgData) -> bool {
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
) -> Box<dyn FnOnce(Receiver<MsgData>, SyncSender<MsgData>) + Send> {
    Box::new(
        move |receiver: Receiver<MsgData>, sender: SyncSender<MsgData>| {
            //-----------------------------------------------------------------------------
            let mut net_msg_cb_fn = |cid: u64, vec_msg: Vec<MsgData>| {
                for mut msg in vec_msg {
                    msg.uid = cid;
                    match sender.try_send(msg) {
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
            let mut msg_kind_cb_fn = |cid: u64, spid: SProtoId| {
                match sender.try_send(MsgData::new_uid_pid(cid, spid as u16)) {
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
            let mut tcp_listen_service: TcpListenService<WanTcpRw, MsgData>;
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
                        Ok(msg_data) => {
                            if msg_data.pid == SProtoId::Disconnect as u16 {
                                tcp_listen_service.del_tcp_socket(msg_data.uid);
                            }else{
                                //这里要优化 判断是否广播消息
                                tcp_listen_service.write_msg(msg_data.uid, msg_data);
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
