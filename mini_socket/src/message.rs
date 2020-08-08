#[derive(Debug, PartialEq)]
pub enum ErrMsg {
    /// 新建一个链接
    NewSocket = 0,

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

/*


/// Msg Id最大值
pub const MSG_MAX_ID: u16 = 4095;

/// data size数据最大字节数
pub const MSG_MAX_SIZE: u32 = 1024 * 1024;

///数据包头长度4个字节
/// msg id: 0 ~ 4095
/// data size: 0 ~ (1024 * 1024)
///|data size:13~32位|+|MID:1~12位|
pub const MSG_HEAD_SIZE: usize = 4;

pub enum MsgEnum {
    SysMsg(SysMsg),
    NetMsg(NetMsg),
}

pub struct SysMsg {
    pub sid: u64,
    pub smid: SysMsgId,
}

pub struct NetMsg {
    pub sid: u64,
    pub data: Vec<u8>,
}

/// 系统间的消息
#[derive(Debug, PartialEq)]
pub enum SysMsgId {
    /// 新建一个链接
    NewSocket = 0,
    /// 断开网络
    CloseSocket = 1,
    /// 网络已断开
    SocketClose = 2,
    /// 服务繁忙
    BusyServer = 3,
    /// 服务器异常
    ExceptionServer = 4,

    SysMsgIdMaxValue = 255,
}
*/
/*
///数据包头长度 10 个字节
/// msg id: 0~4095
/// proto id: 0~65535
/// data size: 0~1024 * 1024
/// extend:可用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
///|data size:13~32位|+|MID:1~12位| + |PID(16位)| + |EXT(32位)
pub const MSG_HEAD_SIZE: usize = 10;
*/

/*
///proto id 从头部数据从第4个字节开始获取
pub const HEAD_DATA_PID_POS: usize = 4;

///ext从头部数据从第6个字节开始获取
pub const HEAD_DATA_EXT_POS: usize = 6;

/// 局域网通信消息
#[derive(Debug)]
pub struct LanMsg {
    /// ser id
    pub sid: u64,
    pub msg: NetMsg,
}

/// 网络基本消息
#[derive(Debug)]
pub struct NetMsg {
    /// conn id
    /// user id
    pub ucid: u64,
    /// msg data
    pub data: Box<MsgData>,
}

#[derive(Debug)]
pub struct MsgData {
    /// 协议Id
    pub pid: u16,
    /// 扩展数据
    pub ext: u32,
    ///消息数据
    pub data: Vec<u8>,
}
*/
