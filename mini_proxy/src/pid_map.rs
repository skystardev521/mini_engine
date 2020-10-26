/// 根据协议Id 映射到 对应服务
use std::collections::HashMap;

pub struct PIdMap {
    /// 可以优化改成数组
    /// pid(协议) sid(服务id)
    pub pid_sid: HashMap<u16, u16>,
}

impl PIdMap {

    pub fn new()->Self{
        PIdMap{pid_sid: HashMap::new()}
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

#[test]
fn test_del(){
    let mut map = PIdMap::new();
    map.add_sid_pid(1, vec![1,2,3,4,5]);
    map.add_sid_pid(2, vec![6,7,8,9,10]);
    map.add_sid_pid(3, vec![11,12,13,14,15]);

    map.del_sid(2);

    map.pid_sid.retain(|key,val|{
        println!("key:{} val:{}", key ,val);
        true
    });

    let array = Box::new([0u16; u16::MAX as usize]);

    for i in 0..10{
        println!("i:{}", array[i]);
    }

}