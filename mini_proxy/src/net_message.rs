use mini_socket::message::ErrMsg;

pub enum WanMsgEnum {
    ErrMsg(u64, ErrMsg),
    NetMsg(u64, Vec<u8>),
}
pub enum LanMsgEnum {
    NetMsg(u64, LanNetMsg),
    ErrMsg(u64, LanErrMsg),
}

pub struct LanNetMsg {
    pub sid: u64,
    pub data: Vec<u8>,
}

pub struct LanErrMsg {
    pub sid: u64,
    pub data: ErrMsg,
}
