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
                Some(msg_data) => {
                    if SProtoId::exists(msg_data.pid){
                        self.wan_sproto_id(SProtoId::new(msg_data.pid), msg_data);
                    }else{
                        //self.sender_lan(msg_data);
                        // todo test code
                        self.wan_service.sender(msg_data);
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
                Some(mut srv_msg) => {
                    if SProtoId::exists(srv_msg.msg.pid){
                        self.lan_sproto_id(SProtoId::new(srv_msg.msg.pid), srv_msg);
                    }else{
                        if let Some(cid) = self.mucid_route.uid_to_cid(srv_msg.msg.uid){
                            self.wan_service.sender({srv_msg.msg.uid = *cid; srv_msg.msg});
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

    fn sender_lan(&self, mut msg_data: MsgData) {
        match self.mucid_route.cid_to_uid(msg_data.uid){
            Some(&0)=>{
                warn!("AuthRequest Unfinished cid:{}", msg_data.uid);
            }Some(uid)=>{
                msg_data.uid = *uid;
                // 要根据 协议id 判断 发送到那个 sid
                match self.mucid_route.get_sid(msg_data.pid, *uid){
                    Some(sid)=>{
                        self.lan_service.sender(SrvMsg::new(sid, msg_data));
                    }
                    None=>{
                        error!("proto id:{} no server handle", msg_data.pid);
                    }
                }
            }
            None=>{
                debug!("AuthRequest Unfinished Or Disconnect cid:{}", msg_data.uid);
            }
        }
    }

    fn  lan_sproto_id(&mut self, spid: SProtoId, mut srv_msg: SrvMsg){
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
                    //通知客户端数据异常
                    let mut wan_msg = srv_msg.msg.clone();
                    self.wan_service.sender({wan_msg.uid = *cid; wan_msg});
                    //通知网络线程断开网络链接
                    self.wan_service.sender(MsgData::new_uid_pid(*cid, SProtoId::Disconnect as u16));
                    //然后再通知其它服务 用户已断线
                    if let Some(vec_sid) = self.mucid_route.get_vec_sid(spid as u16){
                        for sid in vec_sid.iter(){
                            self.lan_service.sender({srv_msg.id = *sid; srv_msg.clone()});
                        }
                    }
                }else{
                    debug!("Disconnect unknown uid:{}", srv_msg.msg.uid)
                }
            },
            SProtoId::ServerBusy=> {
                error!("ServerBusy: uid:{}", srv_msg.msg.uid);
            },
            SProtoId::MsgQueueFull=>  {
                error!("MsgQueueFull: uid:{}", srv_msg.msg.uid);
            },
            SProtoId::ServerRunExc=> {
                error!("ServerRunExc: uid:{}", srv_msg.msg.uid);
            },

            SProtoId::AuthReqPass=> {
                match self.mucid_route.cid_to_uid(srv_msg.msg.uid){
                    Some(zero) =>{
                        if srv_msg.msg.buf.len() < 8{
                            error!("AuthReqPass buffer data error");
                        }
                        let uid = bytes::read_u64(&srv_msg.msg.buf);
                        if zero == &0 {
                            if uid == 0{
                                //登录验证成功后 uid 不能会0
                                error!("AuthReqPass uid is 0 Cannot be equal to 0");
                            }else{
                                //登录验证成功后 把cid uid 保存到mucid_route中
                                self.mucid_route.add_cid_uid(srv_msg.msg.uid, uid);
                            }
                        }else {
                            error!("repeat recv AuthReqPass uid:{}", uid);
                        }
                    }
                    None=>{
                        //网络已端开
                        debug!("Disconnect unknown cid:{}", srv_msg.msg.uid);
                    }
                }
            },

            SProtoId::AuthNotPass=> {
                // 要记录当前连接验证不通过次数
                // 超过一定次数直接断开这个连接
                self.wan_service.sender(srv_msg.msg);
            },
            
            _=> {error!("unknown SProtoId:{:?}", srv_msg.msg.pid)},
        }

    }
    fn wan_sproto_id(&mut self, spid: SProtoId, msg: MsgData){
        match spid {
            SProtoId::Disconnect=> {
                if let Some(uid) = self.mucid_route.cid_to_uid(msg.uid){
                    let hash_id = if *uid > 0 {
                        *uid  //已认证成功的连接
                    }else{
                        msg.uid //未认证成功,未认证完成，没有认证的连接
                    };
                    if let Some(sid) = self.mucid_route.get_sid(msg.pid, hash_id){

                        let msg = MsgData::new_uid_pid(*uid, msg.pid);

                        self.lan_service.sender(SrvMsg::new(sid, msg));

                    }else{
                        error!("proto id:{:?} no server handle", spid);
                    }
                }else{
                    debug!("Disconnect unknown cid:{}", msg.uid)
                }
            }
            SProtoId::AuthRequest=> {
                match self.mucid_route.get_sid(spid as u16, msg.uid){
                    Some(sid)=>{
                        self.mucid_route.add_cid(msg.uid);
                        self.lan_service.sender(SrvMsg::new(sid, msg));
                    }
                    None=>{
                        error!("proto id:{:?} no server handle", spid);
                    }
                }
            }
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
}
