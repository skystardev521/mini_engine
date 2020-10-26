/// 每个连接的第一个包做身份认识
use std::collections::HashMap;

pub struct IdMap {
    /// 认识服id
    //cid: u64,
    /// 认证后的 conn id -->user id
    cid_uid: HashMap<u32, u64>,
    /// 认证后的 user id-->conn id
    uid_cid: HashMap<u64, u32>,

    /// 可以优化改成数组
    /// pid(协议) sid(服务id)
    pub pid_sid: HashMap<u16, u16>,
}

impl IdMap {
    pub fn new() -> Self {
        IdMap {
            cid_uid: HashMap::new(),
            uid_cid: HashMap::new(),
            pid_sid: HashMap::new(),
        }
    }

    #[inline]
    pub fn add_cid_uid(&mut self, cid: u32, uid: u64) {
        self.cid_uid.insert(cid, uid);
        self.uid_cid.insert(uid, cid);
    }

    #[inline]
    pub fn cid_to_uid(&self, cid: u32) -> Option<&u64> {
        self.cid_uid.get(&cid)
    }

    #[inline]
    pub fn uid_to_cid(&self, uid: u64) -> Option<&u32> {
        self.uid_cid.get(&uid)
    }

    #[inline]
    pub fn del_uid_data(&mut self, uid: u64) -> bool {
        match self.uid_cid.remove(&uid) {
            Some(cid) => {
                self.cid_uid.remove(&cid);
                true
            }
            None => false,
        }
    }

    #[inline]
    pub fn del_cid_data(&mut self, cid: u32) -> bool {
        match self.cid_uid.remove(&cid) {
            Some(uid) => {
                self.uid_cid.remove(&uid);
                true
            }
            None => false,
        }
    }

    #[inline]
    /// sid(服务id)  
    pub fn del_sid(&mut self, sid:u16){
        self.pid_sid.retain(|_, val|{
            *val != sid
        })
    }

    #[inline]
    /// sid(服务id)  vec_pid(协议id列表)
    pub fn add_sid_pid(&mut self, sid:u16, vec_pid: Vec<u16>){
        for pid in vec_pid {
            self.pid_sid.insert(pid, sid);
        }
    }

    #[inline]
    /// 根据协议Id获取服务Id
    pub fn get_sid(&self, pid: u16)->Option<&u16>{
        self.pid_sid.get(&pid)
    }
}
