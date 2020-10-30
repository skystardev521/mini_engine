use mini_socket::exc_kind::SProtoId;

pub mod lan {
    pub enum MsgEnum {
        /// 局域网 conn id
        NetMsg(u32, NetMsg),
        /// 局域网 socket id
        ExcMsg(u32, super::SProtoId),
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
        pub ekd: super::SProtoId,
    }
    */
}

pub mod wan {
    pub enum MsgEnum {
        /// 外网 conn id
        NetMsg(u32, NetMsg),
        /// 外网 conn id
        ExcMsg(u32, super::SProtoId),
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
