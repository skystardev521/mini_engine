pub struct Config {
    pub single_read_msg_max_num: u16,
}
pub struct ConfigBuilder {
    single_read_msg_max_num: u16,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            single_read_msg_max_num: 256,
        }
    }
    pub fn builder(&self) -> Config {
        Config {
            single_read_msg_max_num: self.single_read_msg_max_num,
        }
    }
}
