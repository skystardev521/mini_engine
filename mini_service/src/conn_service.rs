use mini_socket::message::NetMsg;
use mini_socket::message::ProtoId;
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
    worker: Worker<NetMsg, ()>,
}

impl ConnService {
    pub fn new(
        workers_config: &WorkerConfig,
        vec_tcp_connect_config: Vec<TcpConnectConfig>,
    ) -> Result<Self, String> {
        let max_task_num = workers_config.get_single_max_task_num();
        let worker = Worker::new(
            String::from("ConnService"),
            workers_config.get_stack_size(),
            workers_config.get_channel_size(),
            worker_closure(max_task_num, vec_tcp_connect_config),
        )?;

        Ok(ConnService { worker: worker })
    }

    #[inline]
    pub fn receiver(&self) -> Option<NetMsg> {
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
    pub fn sender(&self, msg: NetMsg) -> bool {
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
    single_write_msg_max_num: u16,
    vec_tcp_connect_config: Vec<TcpConnectConfig>,
) -> Box<dyn FnOnce(Receiver<NetMsg>, SyncSender<NetMsg>) + Send> {
    Box::new(
        move |receiver: Receiver<NetMsg>, sender: SyncSender<NetMsg>| {
            //-----------------------------------------------------------------------------
            let mut net_msg_cb = |net_msg: NetMsg| {
                match sender.try_send(net_msg) {
                    Ok(()) => return Ok(()),
                    Err(TrySendError::Full(_)) => {
                        error!("TcpConnectService try_send Full");
                        return Err(ProtoId::BusyServer);
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        error!("TcpConnectService try_send Disconnected");
                        return Err(ProtoId::ExceptionServer);
                    }
                };
            };
            //-----------------------------------------------------------------------------
            let mut tcp_connect_service: TcpConnectService;
            match TcpConnectService::new(vec_tcp_connect_config, &mut net_msg_cb) {
                Ok(service) => tcp_connect_service = service,
                Err(err) => {
                    error!("TcpConnectService::new error:{}", err);
                    return;
                }
            }
            //-----------------------------------------------------------------------------
            let mut epoll_wait_timeout = 0;
            let mut single_write_msg_count;
            loop {
                tcp_connect_service.tick();

                match tcp_connect_service.epoll_event(epoll_wait_timeout) {
                    Ok(0) => epoll_wait_timeout = 1,
                    Ok(epevs) => {
                        if epevs == tcp_connect_service.get_epoll_max_events() {
                            epoll_wait_timeout = 0;
                        }
                    }
                    Err(err) => {
                        error!("TcpConnectService epoll_event:{}", err);
                        break;
                    }
                }
                //-----------------------------------------------------------------------------
                single_write_msg_count = 0;
                loop {
                    match receiver.try_recv() {
                        Ok(net_msg) => {
                            tcp_connect_service.write_net_msg(net_msg);
                            single_write_msg_count += 1;
                            if single_write_msg_count == single_write_msg_max_num {
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
