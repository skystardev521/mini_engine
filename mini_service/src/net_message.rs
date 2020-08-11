use mini_socket::message::ErrMsg;
pub enum MsgEnum {
    ErrMsg(u64, ErrMsg),
    NetMsg(u64, Box<NetMsg>),
}

pub struct NetMsg {
    pub sid: u64,
    pub data: Vec<u8>,
}
