/// Msg Id最大值
pub const MSG_MAX_ID: u16 = 4095;

/// data size数据最大字节数
pub const MSG_MAX_SIZE: u32 = 1024 * 1024;

///数据包头长度 10 个字节
/// msg id: 0~4095
/// proto id: 0~65535
/// data size: 0~1024 * 1024
/// extend:可用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
///|data size:13~32位|+|MID:1~12位| + |PID(16位)| + |EXT(32位)
pub const MSG_HEAD_SIZE: usize = 10;

///proto id 从头部数据从第4个字节开始获取
pub const HEAD_DATA_PID_POS: usize = 4;

///ext从头部数据从第6个字节开始获取
pub const HEAD_DATA_EXT_POS: usize = 6;
