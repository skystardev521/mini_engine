use std::time::Duration;

#[derive(Clone, Debug)]
pub struct WConfig {
    name: String,
    /// defalut:0
    stack_size: usize,
    /// defalut:163840
    channel_size: u32,
    /// defalut:1024
    single_max_task_num: u16,
    /// defalut: 1 milis
    sleep_duration: Duration,
}

impl WConfig {
    pub fn new() -> Self {
        WConfig {
            stack_size: 0,
            channel_size: 163840,
            single_max_task_num: 1024,
            name: String::from("WConfig"),
            sleep_duration: Duration::from_millis(1),
        }
    }

    /// 线程的栈大小 0:使用系统默认大小
    pub fn get_stack_size(&self) -> usize {
        self.stack_size
    }
    /// 每个worker间通信任务队列数量
    pub fn get_channel_size(&self) -> u32 {
        self.channel_size
    }
    /// 每个worker单次处理最大任务数量
    pub fn get_single_max_task_num(&self) -> u16 {
        self.single_max_task_num
    }

    /// 空闲时worker休眠时长(毫秒)
    pub fn get_sleep_duration(&self) -> Duration {
        self.sleep_duration
    }

    /// 线程的栈大小 0:使用系统默认大小
    pub fn set_stack_size(&mut self, num: usize) -> &mut Self {
        self.stack_size = num;
        self
    }

    /// 每个worker间通信任务队列数量
    pub fn set_channel_size(&mut self, num: u32) -> &mut Self {
        self.channel_size = if num < 1024 { 1024 } else { num };
        self
    }

    /// 空闲时worker休眠时长(毫秒)
    pub fn set_sleep_duration(&mut self, num: u16) -> &mut Self {
        let n = if num == 0 { 1 } else { num } as u64;
        self.sleep_duration = Duration::from_millis(n);
        self
    }

    /// 每个worker单次处理最大任务数量
    pub fn set_single_max_task_num(&mut self, num: u16) -> &mut Self {
        self.single_max_task_num = if num < 512 { 512 } else { num };
        self
    }
}
