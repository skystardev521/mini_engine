/// 每个连接的第一个包做身份认识
use std::collections::HashMap;

pub struct NetAuth {
    /// 认识服id
    //sid: u64,
    /// 认证后的 net id -->user id
    sid_uid: HashMap<u64, u64>,
    /// 认证后的 user id-->net
    uid_sid: HashMap<u64, u64>,
}


impl NetAuth {

    pub fn new()->Self{
        NetAuth{sid_uid: HashMap::new(), uid_sid: HashMap::new()}
    }

    #[inline]
    pub fn add_sid_uid(&mut self, sid:u64, uid:u64){
        self.sid_uid.insert(sid, uid);
        self.uid_sid.insert(uid, sid);
    }

    #[inline]
    pub fn sid_to_uid(&self,sid: u64)->Option<&u64>{
        self.sid_uid.get(&sid)
    }

    #[inline]
    pub fn uid_to_sid(&self, uid: u64)->Option<&u64>{
        self.uid_sid.get(&uid)
    }

    #[inline]
    pub fn del_uid_data(&mut self, uid: u64)->bool{
        match self.uid_sid.remove(&uid){
            Some(sid)=>
            {
                self.sid_uid.remove(&sid);
                true
            }
            None=> false
        }
    }

    #[inline]
    pub fn del_sid_data(&mut self, sid: u64)->bool{
        match self.sid_uid.remove(&sid){
            Some(uid)=>
            {
                self.sid_uid.remove(&uid);
                true
            }
            None=> false
        }
    }

}