/*
pub enum NetMsg{
    /// 正常消息
    /// 连接id  网络数据
    NorMsg(u64, MsgData),
    /// 异常消息
    /// 连接id   异常kind
    ExcMsg(u64, SProtoId),
}
*/

#[derive(Clone)]
pub struct SrvMsg{
    pub id: u64,
    pub msg: MsgData
}

impl SrvMsg {
    #[inline]
    pub fn new(id:u64, msg:MsgData)->Self{
        SrvMsg{id, msg}
    }


}

/// ext用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
#[derive(Clone)]
pub struct MsgData {
    /// 用户协议id
    pub pid: u16,
    /// 保存扩展数据
    pub ext: u32,
    /// 链接Id用户id
    /// 用户Id不能为0
    pub uid: u64,
    /// 协议对应数据
    pub buf: Vec<u8>,
}


impl MsgData {
    
    pub fn get_uid(&self)->u64{
        self.uid
    }

    #[inline]
    /// pid(协议id)
    pub fn new_pid(pid: u16)->Self{
        MsgData{uid:0, pid, ext:0, buf: vec![]}   
    }

    #[inline]
    /// uid(链接Id用户id),pid(协议id)
    pub fn new_uid_pid(uid: u64, pid: u16)->Self{
        MsgData{uid, pid, ext:0, buf: vec![]}   
    }
}

/// 网络系统协议
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SProtoId {
    /// 有服务加入
    ServerJoin = 0,

    /// 有服务退出
    ServerExit = 1,

     /// 网络连接认证请求
     /// 如果需要Ip把ip加到最后
     /// 或增加一条获取Id的协议
    AuthRequest = 2,

    /// 网络连接认证通过
    /// MsgData.uid(连接Id)
    /// MsgData.buf(用户Id)
    AuthReqPass = 3,
   
    /// 网络连接认证没有通过
    /// MsgData.uid(连接Id)
    AuthNotPass = 4,
    
    /// 断开网络或网络已断开
    Disconnect = 5,

    /// 用户数据异常
    /// 需求断开连接
    ExcUserData = 6,

    /// 服务繁忙
    /// 通常是服务处理不完消息
    ServerBusy = 7,

     /// 服务运行异常
     /// 通常服务访问(DB,Redis)出现问题
    ServerRunExc = 8,

    /// 消息队列已满
    /// 线程或服务繁忙
    MsgQueueFull = 9,
        
    EnumMaxValue = 255,
}


impl SProtoId {
    #[inline]
    pub fn exists(v: u16)-> bool {
        v < Self::EnumMaxValue as u16
    }

    #[inline]
    pub fn new(v: u16)-> Self{
        match v {
            0=> Self::ServerJoin,
            1=> Self::ServerExit,
            2=> Self::AuthRequest,
            3=> Self::AuthReqPass,
            4=> Self::AuthNotPass,
            5=> Self::Disconnect,
            6=> Self::ExcUserData,
            7=> Self::ServerBusy,
            8=> Self::ServerRunExc,
            9=> Self::MsgQueueFull,
            _=> Self::EnumMaxValue,
        }
    }
}