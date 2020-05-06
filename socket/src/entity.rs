pub struct NetData {
    pub token: u64,
    pub task: NetTask,
}

pub struct NetTask {
    //协议Id
    pub id: u16,
    //协议数据
    pub buffer: Vec<u8>,
}

///Max Value 255
pub enum Task_Id {
    New_Client = 0,
    Close_Client = 1,
    Client_Close = 2,
}
