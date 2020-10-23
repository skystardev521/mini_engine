use mini_socket::exc_kind::ExcKind;

pub mod lan {
    pub enum MsgEnum {
        /// 连接id
        NetMsg(u64, NetMsg),
        /// 连接id
        ExcMsg(u64, super::ExcKind),
        //ExcMsg(u64, ExcMsg),
        
    }
    /// ext用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
    pub struct NetMsg {
        /// 用户id
        pub uid: u64,
        /// 用户协议id
        pub pid: u16,
        /// 保存扩展数据
        pub ext: u32,
        /// 协议对应数据
        pub data: Vec<u8>,
    }

    /*
    /// 系统内部消息枚举
    pub struct ExcMsg {
        /// 用户id
        pub uid: u64,
        pub ekd: super::ExcKind,
    }
    */
}

pub mod wan {
    pub enum MsgEnum {
        NetMsg(u64, NetMsg),
        ExcMsg(u64, super::ExcKind),
    }

    /// ext用于：第1位加密，第2位压缩,3~12协议版本，13~32位事务id
    pub struct NetMsg {
        /// 用户协议id
        pub pid: u16,
        /// 保存扩展数据
        pub ext: u32,
        /// 协议对应数据
        pub data: Vec<u8>,
    }
}
