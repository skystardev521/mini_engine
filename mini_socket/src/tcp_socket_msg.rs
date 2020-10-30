pub enum NetMsg{
    /// 正常消息
    /// 连接id  网络数据
    NorMsg(u64, MsgData),
    /// 异常消息
    /// 连接id   异常kind
    ExcMsg(u64, SProtoId),
}

/// ext用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
pub struct MsgData {
    /// 用户协议id
    pub pid: u16,
    /// 保存扩展数据
    pub ext: u32,
    /// 链接对应的用户id
    pub uid: u64,
    /// 协议对应数据
    pub buf: Vec<u8>,
}

/// 网络系统协议
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SProtoId {
    /// 新的Server
    NewServer = 0,
    
    /// 断开网络
    CloseSocket = 1,

    /// 网络已断开
    SocketClose = 2,

    /// 服务繁忙
    BusyServer = 3,

    /// 消息队列已满
    MsgQueueIsFull = 4,

    /// 服务器异常
    ExceptionServer = 5,

    /// 连接id不存在
    SocketIdNotExist = 6,

    /// 网络连接认证通过
    SocketAuthPass = 7,

    /// 网络连接认证没有通过
    SocketAuthNotPass = 8,

    SysMsgIdMaxValue = 255,
}


impl SProtoId {
    
    #[inline]
    pub fn exists(v: u16)-> Option<Self>{
        match v {
            0=> Some(SProtoId::NewServer),
            1=> Some(SProtoId::CloseSocket),
            2=> Some(SProtoId::SocketClose),
            3=> Some(SProtoId::BusyServer),
            4=> Some(SProtoId::MsgQueueIsFull),
            5=> Some(SProtoId::ExceptionServer),
            6=> Some(SProtoId::SocketIdNotExist),
            7=> Some(SProtoId::SocketAuthPass),
            8=> Some(SProtoId::SocketAuthNotPass),
            _=> None,
        }
    }
}