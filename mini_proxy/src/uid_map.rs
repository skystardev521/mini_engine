/// 每个连接的第一个包做身份认识
use std::collections::HashMap;

pub struct UIdMap {
    /// 认识服id
    //cid: u64,
    /// 认证后的 conn id -->user id
    cid_uid: HashMap<u32, u64>,
    /// 认证后的 user id-->conn id
    uid_cid: HashMap<u64, u32>,
}

impl UIdMap {
    pub fn new() -> Self {
        UIdMap {
            cid_uid: HashMap::new(),
            uid_cid: HashMap::new(),
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
}
