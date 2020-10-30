pub enum NetMsg{
    /// 正常消息
    /// 连接id  网络数据
    NorMsg(u64, MsgData),
    /// 异常消息
    /// 连接id   异常kind
    ExcMsg(u64, NetSMP),
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

impl MsgData {
    pub fn new(pid: u16)->Self{
        MsgData{
            pid,
            ext:0,
            uid:0,
            buf:vec![]
        }
    }
}


/// 网络 系统消息协议
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NetSMP {
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

    /// Socket Id not exist
    SocketIdNotExist = 6,

    SysMsgIdMaxValue = 255,
}



impl NetSMP {
    
    #[inline]
    pub fn is_NetSMP(v: u16)-> bool{
        v < NetSMP::SysMsgIdMaxValue as u16
    }

    #[inline]
    pub fn from(v: u16)->Self{
        match v {
            0=> NetSMP::NewServer,
            1=> NetSMP::CloseSocket,
            2=> NetSMP::SocketClose,
            3=> NetSMP::BusyServer,
            4=> NetSMP::MsgQueueIsFull,
            5=> NetSMP::ExceptionServer,
            6=> NetSMP::SocketIdNotExist,
            _=> NetSMP::SysMsgIdMaxValue,
        }
    }
}