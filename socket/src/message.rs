//包ID最大值
pub const MSG_MAX_ID: u16 = 4095;

///数据包头长度 6 个字节
///MsgHead(datasize:13~32位)+(id:1~12位) + dataid(16位)
pub const MSG_HEAD_SIZE: usize = 6;

///数据最大字节数
pub const MSG_MAX_SIZE: u32 = 1024 * 1024;

///客户最大数量
pub const CLIENT_MAX_NUM: u16 = u16::MAX;

pub struct NetMsg {
    /// client id
    pub id: u64,
    pub data: MsgData,
}

pub struct MsgData {
    //协议Id
    pub id: u16,
    //协议数据
    pub data: Vec<u8>,
}

pub struct MsgHead {
    pub id: u16,
    pub dataid: u16,
    pub datasize: u32,
}

///Max Value 255
pub enum MsgDataId {
    NewClient = 0,
    CloseClient = 1,
    ClientClose = 2,

    IdMaxValue = 255,
}
