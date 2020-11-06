/// 每个连接的第一个包做身份认识
use std::collections::HashMap;

pub struct MucIdRoute {
    /// 连接id 转 用户Id
    cid_uid: HashMap<u64, u64>,
    /// 用户Id 转 连接id
    uid_cid: HashMap<u64, u64>,

    /// 可以优化改成数组
    /// mid(协议id) sid(服务id)
    mid_sid: HashMap<u16, Vec<u64>>,
}

impl MucIdRoute {
    pub fn new() -> Self {
        MucIdRoute {
            cid_uid: HashMap::new(),
            uid_cid: HashMap::new(),
            mid_sid: HashMap::new(),
        }
    }

    /// 增加 连接id uid=0
    #[inline]
    pub fn add_cid(&mut self, cid: u64) {
        self.cid_uid.insert(cid, 0);
    }
    
    /// 增加 连接id 与 用户Id
    #[inline]
    pub fn add_cid_uid(&mut self, cid: u64, uid: u64) {
        self.cid_uid.insert(cid, uid);
        self.uid_cid.insert(uid, cid);
    }

    /// 根据连接id 获取 用户Id
    #[inline]
    pub fn cid_to_uid(&self, cid: u64) -> Option<&u64> {
        self.cid_uid.get(&cid)
    }
    
    /// 根据用户Id 获取 连接id
    #[inline]
    pub fn uid_to_cid(&self, uid: u64) -> Option<&u64> {
        self.uid_cid.get(&uid)
    }

    /// 删除用户Id,连接id
    #[inline]
    pub fn del_uid_cid(&mut self, uid: u64) -> bool {
        match self.uid_cid.remove(&uid) {
            Some(cid) => {
                self.cid_uid.remove(&cid);
                true
            }
            None => false,
        }
    }

    /// 删除连接id,用户Id
    #[inline]
    pub fn del_cid_data(&mut self, cid: u64) -> bool {
        match self.cid_uid.remove(&cid) {
            Some(uid) => {
                self.uid_cid.remove(&uid);
                true
            }
            None => false,
        }
    }

    #[inline]
    /// 根据协议Id, (负载均衡)id, 来获取服务Id
    pub fn get_sid(&self, pid: u16, hash_id: u64)->Option<u64>{
        match self.mid_sid.get(&pid){
            Some(vec_sid)=>{
                Some(vec_sid[(hash_id % (vec_sid.len() as u64)) as usize])
            }
            None=>None
        }
    }

    #[inline]
    /// 根据协议Id, 获取处理这条协议id的 所有服务Id
    pub fn get_vec_sid(&self, pid: u16)->Option<&Vec<u64>>{
        self.mid_sid.get(&pid)
    }

    #[inline]
    /// 增加 sid(服务id) 及 服务支持所有协议  
    pub fn add_sid(&mut self, sid:u64, vec_pid: Vec<u16>){
        for pid in vec_pid {
            match self.mid_sid.get_mut(&pid){
                Some(vec_sid)=>{
                    vec_sid.push(sid);
                }
                None=>{
                    self.mid_sid.insert(pid, vec![sid]);
                }
            }
        }
    }

    #[inline]
    /// 删除sid(服务id) 及 服务的所有协议  
    pub fn del_sid(&mut self, sid:u64){
        self.mid_sid.retain(|_, vec_sid|{
            vec_sid.retain(|&val|{ val != sid });
            0 != vec_sid.len()
        })
    }
}

#[test]
fn test(){
    let mut mucid_route = MucIdRoute::new();

    mucid_route.add_sid(1, vec![1,2,3,4,5]);
    mucid_route.add_sid(11, vec![1,2,3,4,5]);
    mucid_route.add_sid(2, vec![6,7,8,9,10]);
    mucid_route.add_sid(22, vec![6,7,8,9,10]);
    mucid_route.add_sid(3, vec![11,12,13,14,15]);
    mucid_route.add_sid(33, vec![11,12,13,14,15]);

    println!("pid:{} uid:{}, sid:{}", 8, 13, mucid_route.get_sid(8, 13).unwrap());

    mucid_route.del_sid(3);

    /*
    for (key, vec_sid) in mucid_route.mid_sid.iter() {
        for sid in vec_sid.iter(){
            println!("key:{}, vec_sid:{}", key, sid)
        }
    }
    */
}