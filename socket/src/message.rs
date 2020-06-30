//包ID最大值
pub const MSG_MAX_ID: u16 = 4095;

///数据包头长度 6 个字节
///|data size:13~32位|+|id:1~12位| + |data_id(16位)|
pub const MSG_HEAD_SIZE: usize = 6;

///头部数据 id 从第4个字节开始获取
pub const HEAD_DATA_ID_POS: usize = 4;

///数据最大字节数
pub const MSG_MAX_SIZE: u32 = 1024 * 1024;

///客户最大数量
pub const CLIENT_MAX_NUM: u16 = u16::MAX;

pub struct NetMsg {
    /// client id
    pub id: u64,
    pub data: Box<MsgData>,
}

pub struct MsgData {
    //协议Id
    pub id: u16,
    //协议数据
    pub data: Vec<u8>,
}

/// 系统数据Id
/// 用于系统内部通信
pub enum SysDataId {
    NewClient = 0,
    CloseClient = 1,
    ClientClose = 2,

    IdMaxValue = 255,
}
