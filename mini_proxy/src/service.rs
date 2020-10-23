use crate::config::Config;
use crate::lan_service::LanService;
use crate::head_proto::{lan,wan};
use crate::net_auth::NetAuth;
use mini_socket::exc_kind::ExcKind;

use crate::wan_service::WanService;
use log::error;
use mini_utils::bytes;
use std::thread;
use std::time::Duration;

/// 用于把 广域网的数据 转到 局域网服务中
pub struct Service {
    net_auth: NetAuth,
    wan_service: WanService,
    lan_service: LanService,
    single_max_task_num: u16,
    sleep_duration: Duration,
}

impl Drop for Service {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped mini_proxy Service while unwinding");
        } else {
            error!("dropped mini_proxy Service while not unwinding");
        }
    }
}

impl Service {
    pub fn new(config: Config) -> Result<Self, String> {
        let wan_service = WanService::new(&config.worker_config, config.wan_listen_config.clone())?;
        let lan_service = LanService::new(&config.worker_config, config.lan_listen_config.clone())?;

        let sleep_duration = config.worker_config.get_sleep_duration();
        let single_max_task_num = config.worker_config.get_single_max_task_num();

        Ok(Service {
            wan_service,
            lan_service,
            sleep_duration,
            single_max_task_num,
            net_auth: NetAuth::new(),
        })
    }

    pub fn run(&self) {
        loop {
            let mut is_sleep = true;
            if !self.wan_receiver() {
                is_sleep = false;
            }
            if !self.lan_receiver() {
                is_sleep = false;
            }
            if is_sleep {
                thread::sleep(self.sleep_duration);
            }
        }
    }

    /// empty:true data:false
    fn wan_receiver(&self) -> bool {
        let mut num = 0;
        loop {
            match self.wan_service.receiver() {
                None => return true,
                //要把tcp_socket id  转 用户id
                Some(wan::MsgEnum::NetMsg(wan_sid, msg)) => {
                    //sid 连接wan 的sid
                    if let Some(uid) = self.net_auth.sid_to_uid(wan_sid) {
                        let lan_msg = lan::NetMsg{
                            uid:*uid, pid:msg.pid, ext: msg.ext, data:msg.data
                        };

                        // 要根据 协议id 判断 发送到那个 lan_sid

                        self.sender_lan(lan::MsgEnum::NetMsg(lan_sid, lan_msg));
                    }else{

                    }
                    
                    //self.sender_wan(WanMsgEnum::NetMsg(sid, msg));
                    //self.sender_lan(LanMsgEnum::NetMsg(sid, LanNetMsg { sid: sid, msg: msg }));
                }

                //要把tcp_socket id  转 用户id
                Some(wan::MsgEnum::ExcMsg(wan_sid, ekd)) => {
                    if let Some(uid) = self.net_auth.sid_to_uid(wan_sid) {
                        let lan_msg = lan::NetMsg{
                            uid:*uid, pid: ekd as u16, ext: 0, data:vec![]
                        };

                        // 要根据 协议id 判断 发送到那个 lan_sid
                        self.sender_lan(lan::MsgEnum::NetMsg(lan_sid, lan_msg));
                    }else{

                    }
                    //self.sender_wan(WanMsgEnum::MsgKind(sid, kind));
                    //self.sender_lan(LanMsgEnum::MsgKind(sid, LanMsgKind { sid, kind }));
                }
            }
            num += 1;
            if num == self.single_max_task_num {
                return false;
            }
        }
    }

    /// empty:true data:false
    fn lan_receiver(&self) -> bool {
        let mut num = 0;
        loop {
            match self.lan_service.receiver() {
                None => return true,
                Some(lan::MsgEnum::NetMsg(sid, msg)) => {
                    //sid 对应服务连接id
                    //要把 用户id 转 tcp_socket id
                    if let Some(wan_sid) = self.net_auth.uid_to_sid(msg.uid){
                         // 判断 
                        if ExcKind::is_exckind(msg.pid){
                            let ekd = ExcKind::from(msg.pid);
                            self.sender_wan(wan::MsgEnum::ExcMsg(*wan_sid, ekd));
                        }else{
                            let wan_msg = wan::NetMsg{
                                pid:msg.pid,ext:msg.ext, data:msg.data
                            };
                            self.sender_wan(wan::MsgEnum::NetMsg(*wan_sid, wan_msg));
                        }
                    }else{

                    }
                   
                    num += 1;
                    if num == self.single_max_task_num {
                        return false;
                    }
                }

                //要把 用户id 转 tcp_socket id
                Some(lan::MsgEnum::ExcMsg(sid, ekd)) => {
                    // 局域网内 网络异常
                    /*
                    let wan_sid = self.auth.uid_to_sid(msg.uid);
                    self.sender_wan(wan::MsgEnum::ExcMsg(wan_sid, ekd));
                    */
                    num += 1;
                    if num == self.single_max_task_num {
                        return false;
                    }
                }
            }
        }
    }

    fn sender_wan(&self, msg: wan::MsgEnum) {
        self.wan_service.sender(msg);
    }

    fn sender_lan(&self, msg: lan::MsgEnum) {
        self.lan_service.sender(msg);
    }

    fn decode_id(buffer: &Vec<u8>) -> Result<u16, &str> {
        if buffer.len() < 2 {
            Err("data len is 0")
        } else {
            Ok(bytes::read_u16(&buffer))
        }
    }

    fn encode(buffer: &Vec<u8>) {}

    /*
    fn encode(ext: u32) -> Vec<u8> {
        let str = "0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz0123456789AaBbCcDdEdFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz";
        let len = 2 + 4 + 250;
        let mut buffer: Vec<u8> = vec![0u8; len];
        bytes::write_u16(&mut buffer, 123);
        bytes::write_u32(&mut buffer[2..], ext);
        bytes::write_bytes(&mut buffer[6..], &str.as_bytes()[0..250]);
        //warn!("encode buffer len:{} ext:{}", buffer.len(), ext);
        buffer
    }

    fn decode(buffer: &Vec<u8>) -> (u16, u32, Vec<u8>) {
        //warn!("decode buffer len:{}", buffer.len());
        let pid = bytes::read_u16(&buffer);
        let ext = bytes::read_u32(&buffer[2..]);
        let data = bytes::read_bytes(&buffer[6..]);
        (pid, ext, data)
    }
    */
}
