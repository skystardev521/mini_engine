use crate::exc_kind::ExcKind;

pub enum NetMsg{
    /// 正常消息
    NorMsg(u64, MsgData),
    /// 异常消息
    ExcMsg(u64, ExcKind),
}

/// ext用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
pub struct MsgData {
    /// 用户id
    /// 连接id
    pub uid: u64,
    /// 用户协议id
    pub pid: u16,
    /// 保存扩展数据
    pub ext: u32,
    /// 协议对应数据
    pub buf: Vec<u8>,
}
