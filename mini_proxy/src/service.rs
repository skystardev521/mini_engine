use crate::config::Config;
use crate::lan_service::LanService;
use crate::mucid_route::MucIdRoute;
use mini_socket::tcp_socket_msg::{SrvMsg, MsgData, SProtoId};

use crate::wan_service::WanService;
use log::{error,warn,debug};
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
                Some(msg) => {
                    if SProtoId::exists(msg.pid){
                        self.handle_wan_spid(SProtoId::new(msg.pid), msg);
                    }else{
                        let hash_val;
                        let mut mut_msg = msg;
                        if let Some(uid) = self.mucid_route.cid_to_uid(msg.uid){
                            hash_val = *uid;
                            (mut_msg).uid = *uid;
                        }else{
                            hash_val = msg.uid;

                             // todo temp code 
                             self.mucid_route.add_cid_uid(msg.uid, msg.uid);

                            // 新连接发过来的第一个包没有 uid
                            self.mucid_route.add_cid_uid(msg.uid, 0);

                        }
                        // todo temp code
                        self.sender_wan(msg);
                        /*
                        // 要根据 协议id 判断 发送到那个 sid
                        match self.mucid_route.get_sid(mut_msg.pid, hash_val){
                            Some(sid)=>{
                                self.sender_lan(NetMsg::NorMsg(sid, mut_msg));
                            }
                            None=>{
                                error!("proto id:{} no server handle", mut_msg.pid);
                            }
                        }
                        */
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
                Some(srv_msg) => {
                    if SProtoId::exists(srv_msg.msg.pid){
                        self.handle_lan_spid(SProtoId::new(srv_msg.msg.pid), srv_msg);
                    }else{
                        if let Some(cid) = self.mucid_route.uid_to_cid(srv_msg.msg.uid){
                            self.sender_wan(srv_msg.msg);
                        }else{
                            debug!("uid_to_cid unknown uid:{}", srv_msg.msg.uid)
                        }
                    }
                   
                    num += 1;
                    if num == self.single_max_task_num {
                        return false;
                    }
                }
            }
        }
    }

    fn sender_wan(&self, msg: MsgData) {
        self.wan_service.sender(msg);
    }

    fn sender_lan(&self, msg: SrvMsg) {
        self.lan_service.sender(msg);
    }

    fn handle_lan_spid(&mut self, spid: SProtoId, srv_msg: SrvMsg){
        match spid {
            SProtoId::ServerJoin=> {
                let vec_pid = Self::get_sid_proto(&srv_msg.msg.buf);
                self.mucid_route.add_sid(srv_msg.id, vec_pid);
            },
            SProtoId::ServerExit=>{
                warn!("Server Id:{} Exit", srv_msg.id);
                self.mucid_route.del_sid(srv_msg.id);
            }
            SProtoId::ExcUserData=> {
                
                if let Some(cid) = self.mucid_route.uid_to_cid(srv_msg.msg.uid){
                    let mut mut_msg = srv_msg;
                    let uid = mut_msg.msg.uid;
                    mut_msg.msg.uid = *cid;
                    //通知客户端数据异常
                    self.sender_wan(mut_msg.msg.clone());
                    //通知网络线程断开网络链接
                    self.sender_wan(MsgData::new_pid(SProtoId::Disconnect as u16));

                    mut_msg.msg.uid = uid;
                    //然后再通知其它服务 用户已断线
                    if let Some(vec_sid) = self.mucid_route.get_vec_sid(spid as u16){
                        for sid in vec_sid.iter(){
                            mut_msg.id = *sid;
                            self.sender_lan(mut_msg.clone());
                        }
                    }
                }else{
                    debug!("Disconnect unknown uid:{}", srv_msg.msg.uid)
                }
            },
            SProtoId::ServerBusy=> {
                debug!("ServerBusy: uid:{}", srv_msg.msg.uid);
            },
            SProtoId::MsgQueueFull=>  {
                debug!("MsgQueueFull: uid:{}", srv_msg.msg.uid);
            },
            SProtoId::ServerRunExc=> {
                debug!("ServerRunExc: uid:{}", srv_msg.msg.uid);
            },

            SProtoId::AuthReqPass=> {
                if srv_msg.msg.buf.len() < 8{
                    error!("AuthReqPass buffer data error");
                }
                let uid = bytes::read_u64(&srv_msg.msg.buf);
                self.mucid_route.add_cid_uid(srv_msg.msg.uid, uid);
            },

            SProtoId::AuthNotPass=> {
                // 要记录当前连接验证不通过次数
                // 超过一定次数直接断开这个连接
                self.sender_wan(srv_msg.msg);
            },
            
            _=> {error!("unknown SProtoId:{:?}", srv_msg.msg.pid)},
        }

    }
    fn handle_wan_spid(&mut self, spid: SProtoId, msg: MsgData){
        match spid {
            SProtoId::Disconnect=> {
                if let Some(uid) = self.mucid_route.cid_to_uid(msg.uid){
                    let hash_id = if *uid > 0 {
                        *uid  //已认证成功的连接
                    }else{
                        msg.uid //未认证成功或没有认证的连接
                    };

                    if let Some(sid) = self.mucid_route.get_sid(msg.pid, hash_id){

                        let msg = MsgData::new_uid_pid(*uid, msg.pid);

                        self.sender_lan(SrvMsg::new(sid, msg));
                    }else{
                        error!("proto id:{:?} no server handle", spid);
                    }
                }else{
                    debug!("Disconnect unknown cid:{}", msg.uid)
                }
            },

            SProtoId::AuthRequest=> {
                if let Some(sid) = self.mucid_route.get_sid(spid as u16, msg.uid){
                    self.sender_lan(SrvMsg::new(sid, msg));
                }else{
                    error!("proto id:{:?} no server handle", spid);
                }
            },
            _=> {error!("unknown SProtoId:{:?}", spid)},
        }
    }

    fn get_sid_proto(buf: &Vec<u8>)->Vec<u16>{
        let mut pos;
        let mut end_pos = 2;
        let buf_size = buf.len();
        let mut vec_u16 = Vec::with_capacity(buf_size / 2);

        while end_pos <= buf_size {
            pos = end_pos; 
            end_pos = end_pos + 2;
            vec_u16.push(bytes::read_u16(&buf[pos..]));
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
