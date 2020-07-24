/// Msg Id最大值
pub const MSG_MAX_ID: u16 = 4095;

/// data size数据最大字节数
pub const MSG_MAX_SIZE: u32 = 1024 * 1024;

///数据包头长度 10 个字节
/// msg id: 0~4095
/// proto id: 0~65535
/// data size: 0~1024 * 1024
/// extend:可用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
///|data size:13~32位|+|MID:1~12位| + |PID(16位)| + |EXT(32位)
pub const MSG_HEAD_SIZE: usize = 10;

///proto id 从头部数据从第4个字节开始获取
pub const HEAD_DATA_PID_POS: usize = 4;

///ext从头部数据从第6个字节开始获取
pub const HEAD_DATA_EXT_POS: usize = 6;

pub struct NetMsg {
    /// socket id
    pub sid: u64,
    /// msg data
    pub data: Box<MsgData>,
}

pub struct MsgData {
    /// 协议Id
    pub pid: u16,
    /// 扩展数据
    pub ext: u32,
    ///消息数据
    pub data: Vec<u8>,
}

/// 基本协议Id
/// 非业务协议
#[derive(Debug, PartialEq)]
pub enum ProtoId {
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

    IdMaxValue = 255,
}
