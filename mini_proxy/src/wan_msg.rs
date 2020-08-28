use mini_socket::exc_kind::ExcKind;
pub enum WanMsg {
    NetMsg(u64, NetMsg),
    ExcMsg(u64, ExcKind),
}

/// ext用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
pub struct NetMsg {
    pub pid: u16,
    /// 保存扩展数据
    pub ext: u32,
    /// 协议对应数据
    pub data: Vec<u8>,
}
