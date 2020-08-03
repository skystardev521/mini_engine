/// 每个连接的第一个包做身份认识
use std::collections::HashMap;

pub struct Auth {
    /// 认识服id
    sid: u64,
    /// 认证后的 net id -->user id
    sid_uid: HashMap<u64, u64>,
    /// 认证后的 user id-->net
    uid_sid: HashMap<u64, u64>,
}
