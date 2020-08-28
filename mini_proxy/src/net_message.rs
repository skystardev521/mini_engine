use mini_socket::exc_kind::ExcKind;

pub enum WanMsgEnum {
    NetMsg(NetMsg),
    ExcMsg(ExcMsg),
}
pub enum LanMsgEnum {
    NetMsg(u64, NetMsg),
    ExcMsg(u64, ExcMsg),
}

pub struct NetMsg {
    pub sid: u64,
    pub pid: u16,
    /// 列如用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
    pub ext: u32,
    pub buf: Vec<u8>,
}

pub struct ExcMsg {
    pub sid: u64,
    pub ekd: ExcKind,
}
