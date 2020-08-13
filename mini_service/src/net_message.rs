use mini_socket::msg_kind::MsgKind;
pub enum MsgEnum {
    NetMsg(u64, NetMsg),
    MsgKind(u64, MsgKind),
}

pub struct NetMsg {
    pub sid: u64,
    pub msg: Vec<u8>,
}
