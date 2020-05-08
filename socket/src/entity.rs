pub struct NetMsg {
    pub token: u64,
    pub net_data: NetData,
}

pub struct NetData {
    //协议Id
    pub id: u16,
    //协议数据
    pub buffer: Vec<u8>,
}

///Max Value 255
pub enum NetDataId {
    NewClient = 0,
    CloseClient = 1,
    ClientClose = 2,
}
