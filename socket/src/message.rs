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
#[derive(PartialEq)]
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
