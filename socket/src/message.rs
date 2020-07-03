pub struct NetMsg {
    /// socket id
    pub id: u64,
    /// msg data
    pub data: Box<MsgData>,
}

pub struct MsgData {
    ///消息Id
    pub id: u16,
    ///消息数据
    pub data: Vec<u8>,
}

/// 系统消息Id
/// 用于系统内部通信
#[derive(PartialEq)]
pub enum MsgDataId {
    /// 新建一个链接
    NewSocket = 0,
    /// 断开网络
    CloseSocket = 1,
    /// 网络已断开
    SocketClose = 2,

    IdMaxValue = 255,
}
