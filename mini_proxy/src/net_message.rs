use mini_socket::msg_kind::MsgKind;

pub enum WanMsgEnum {
    NetMsg(u64, Vec<u8>),
    MsgKind(u64, MsgKind),
}
pub enum LanMsgEnum {
    NetMsg(u64, LanNetMsg),
    MsgKind(u64, LanMsgKind),
}

pub struct LanNetMsg {
    pub sid: u64,
    pub buff: Vec<u8>,
}

pub struct LanMsgKind {
    pub sid: u64,
    pub kind: MsgKind,
}
