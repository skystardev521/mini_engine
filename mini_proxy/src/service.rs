use crate::config::Config;
use crate::lan_service::LanService;
use crate::mucid_route::MucIdRoute;
use mini_socket::tcp_socket_msg::{NetMsg, MsgData, SProtoId};

use crate::wan_service::WanService;
use log::{error,debug};
use mini_utils::bytes;
use std::thread;
use std::time::Duration;

/// 用于把 广域网的数据 转到 局域网服务中
pub struct Service {
    mucid_route: MucIdRoute,
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
        let wan_service = WanService::new(&config.wconfig, config.wan_listen_config.clone())?;
        let lan_service = LanService::new(&config.wconfig, config.lan_listen_config.clone())?;

        let sleep_duration = config.wconfig.get_sleep_duration();
        let single_max_task_num = config.wconfig.get_single_max_task_num();

        Ok(Service {
            wan_service,
            lan_service,
            sleep_duration,
            single_max_task_num,
            mucid_route: MucIdRoute::new(),
        })
    }

    pub fn run(&mut self) {
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
    fn wan_receiver(&mut self) -> bool {
        let mut num = 0;
        loop {
            match self.wan_service.receiver() {
                None => return true,
                //要把tcp_socket id  转 用户id
                Some(NetMsg::NorMsg(cid, msg)) => {

                    let hash_val;
                    let mut mut_msg = msg;
                    if let Some(uid) = self.mucid_route.cid_to_uid(cid){
                        hash_val = *uid;
                        (mut_msg).uid = *uid;
                    }else{
                        hash_val = cid;
                        // 新连接发过来的第一个包没有 uid
                        self.mucid_route.add_cid_uid(cid, 0);
                    }

                    // 要根据 协议id 判断 发送到那个 sid
                    match self.mucid_route.get_sid(mut_msg.pid, hash_val){
                        Some(sid)=>{
                            self.sender_lan(NetMsg::NorMsg(sid, mut_msg));
                        }
                        None=>{
                            error!("proto id:{} no sid", mut_msg.pid);
                        }
                    }
                }

                //要把tcp_socket id  转 用户id
                Some(NetMsg::ExcMsg(wan_sid, ekd)) => {
                    match self.mucid_route.cid_to_uid(wan_sid) {
                        Some(uid)=>{
                            // 要根据 协议id 判断 发送到那个 lan_sid
                            let lan_sid = 1;
                            let msg = MsgData{uid: *uid, pid: ekd as u16, ext:0, buf:vec![]};                        
                            self.sender_lan(NetMsg::NorMsg(lan_sid, msg));
                        }
                        None=>{

                        }
                    }
                }
            }
            num += 1;
            if num == self.single_max_task_num {
                return false;
            }
        }
    }

    
    /// empty:true data:false
    fn lan_receiver(&mut self) -> bool {
        let mut num = 0;
        loop {
            match self.lan_service.receiver() {
                None => return true,
                Some(NetMsg::NorMsg(sid, msg)) => {
                    if let Some(spid) = SProtoId::exists(msg.pid){
                        self.handle_spid_msg(sid, spid, msg);
                    }else{
                        if let Some(cid) = self.mucid_route.uid_to_cid(msg.uid){
                            self.sender_wan(NetMsg::NorMsg(*cid, msg));
                        }else{
                            debug!("uid_to_cid unknown uid:{}", msg.uid)
                        }
                    }
                   
                    num += 1;
                    if num == self.single_max_task_num {
                        return false;
                    }
                }
                //要把 用户id 转 tcp_socket id
                Some(NetMsg::ExcMsg(sid, spid)) => {
                    // 局域网内 网络异常
                    /*
                    let wan_sid = self.auth.uid_to_sid(msg.uid);
                    self.sender_wan(NetMsg::ExcMsg(wan_sid, ekd));
                    */

                    self.mucid_route.del_sid(sid);

                    num += 1;
                    if num == self.single_max_task_num {
                        return false;
                    }
                }
            }
        }
    }

    fn sender_wan(&self, msg: NetMsg) {
        self.wan_service.sender(msg);
    }

    fn sender_lan(&self, msg: NetMsg) {
        self.lan_service.sender(msg);
    }

    fn handle_spid_msg(&mut self, sid: u64, spid: SProtoId, msg: MsgData){
        match spid {
            SProtoId::NewServer=> {
                let vec_pid = Self::get_sid_proto(&msg);
                self.mucid_route.add_sid(sid, vec_pid);
            },
            SProtoId::CloseSocket=> {
                if let Some(cid) = self.mucid_route.uid_to_cid(msg.uid){
                    self.sender_wan(NetMsg::ExcMsg(*cid, spid));
                }else{
                    debug!("CloseSocket unknown uid:{}", msg.uid)
                }
            },
            SProtoId::SocketClose=> {
                debug!("SocketClose uid:{}", msg.uid)
            },
            SProtoId::BusyServer=> {
                debug!("BusyServer: uid:{}", msg.uid);
            },
            SProtoId::MsgQueueIsFull=>  {
                debug!("MsgQueueIsFull: uid:{}", msg.uid);
            },
            SProtoId::ExceptionServer=> {
                debug!("ExceptionServer: uid:{}", msg.uid);
            },
            SProtoId::SocketIdNotExist=> {
                debug!("SocketIdNotExist uid:{}", msg.uid);
            },
            SProtoId::SocketAuthPass=> {
                if msg.buf.len() < 8{
                    error!("SocketAuthPass buf data error");
                }
                let cid = bytes::read_u64(&msg.buf);
                self.mucid_route.add_cid_uid(cid, msg.uid);
            },
            SProtoId::SocketAuthNotPass=> {
                if msg.buf.len() < 8{
                    error!("SocketAuthNotPass buf data error");
                }
                let cid = bytes::read_u64(&msg.buf);
                self.sender_wan(NetMsg::NorMsg(cid, msg));
            },
            
            _=> {error!("unknown SProtoId:{:?}", msg.pid)},
        }
    }

    fn get_sid_proto(msg: &MsgData)->Vec<u16>{
        let mut pos;
        let mut end_pos = 2;
        let buf_size = msg.buf.len();
        let mut vec_u16 = Vec::with_capacity(buf_size / 2);

        while end_pos <= buf_size {
            pos = end_pos; 
            end_pos = end_pos + 2;
            vec_u16.push(bytes::read_u16(&msg.buf[pos..]));
        }
        vec_u16
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
