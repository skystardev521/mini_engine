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
pub enum SysMsgId {
    NewClient = 0,
    CloseClient = 1,
    ClientClose = 2,

    IdMaxValue = 255,
}
