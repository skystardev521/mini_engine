use std::ffi::{CStr, CString};
use std::time::Duration;

#[derive(Clone)]
pub struct Config {
    pub workers_config: WorkersConfig,
    pub vec_connect_config: Vec<ConnConfig>,
}

impl Config {
    pub fn new(workers_config: WorkersConfig, vec_connect_config: Vec<ConnConfig>) -> Self {
        Config {
            workers_config,
            vec_connect_config,
        }
    }
}

#[derive(Clone)]
pub struct WorkersConfig {
    name: String,
    worker_num: u8,
    stack_size: usize,
    channel_size: u16,
    ping_interval: Duration,
    sleep_duration: Duration,
    single_max_task_num: u16,
}

impl WorkersConfig {
    pub fn new() -> Self {
        WorkersConfig {
            worker_num: 2,
            stack_size: 0,
            channel_size: 1024,
            single_max_task_num: 1024,
            name: String::from("mysql"),
            ping_interval: Duration::from_secs(300),
            sleep_duration: Duration::from_millis(1),
        }
    }

    /// 设置worker数量
    pub fn get_worker_num(&self) -> u8 {
        self.worker_num
    }
    /// 线程的栈大小 0:使用系统默认大小
    pub fn get_stack_size(&self) -> usize {
        self.stack_size
    }
    /// 每个worker间通信任务队列数量
    pub fn get_channel_size(&self) -> u16 {
        self.channel_size
    }
    /// 每个worker单次处理最大任务数量
    pub fn get_single_max_task_num(&self) -> u16 {
        self.single_max_task_num
    }
    /// ping mysql connect 间隔 单位sec
    pub fn get_ping_interval(&self) -> Duration {
        self.ping_interval
    }

    /// 空闲时worker休眠时长
    pub fn get_sleep_duration(&self) -> Duration {
        self.sleep_duration
    }

    /// 设置worker数量
    pub fn set_worker_num(&mut self, num: u8) -> &mut Self {
        self.worker_num = if num == 0 { 1 } else { num };
        self
    }

    /// 线程的栈大小 0:使用系统默认大小
    pub fn set_stack_size(&mut self, num: usize) -> &mut Self {
        self.stack_size = num;
        self
    }

    /// 每个worker间通信任务队列数量
    pub fn set_channel_size(&mut self, num: u16) -> &mut Self {
        self.channel_size = if num < 128 { 128 } else { num };
        self
    }

    /// 每个worker单次处理最大任务数量
    pub fn set_single_max_task_num(&mut self, num: u16) -> &mut Self {
        self.single_max_task_num = if num < 128 { 128 } else { num };
        self
    }

    /// ping mysql connect 间隔 单位sec
    pub fn set_ping_interval(&mut self, num: u16) -> &mut Self {
        let n = if num == 0 { 1 } else { num } as u64;
        self.ping_interval = Duration::from_secs(n);
        self
    }

    /// 空闲时worker休眠时长
    pub fn set_sleep_duration(&mut self, num: u16) -> &mut Self {
        let n = if num == 0 { 1 } else { num } as u64;
        self.sleep_duration = Duration::from_millis(n);
        self
    }
}

#[derive(Clone)]
pub struct ConnConfig {
    port: u16,
    user: Option<CString>,
    host: Option<CString>,
    password: Option<CString>,
    database: Option<CString>,
    unix_socket: Option<CString>,
}

impl ConnConfig {
    pub fn new() -> Self {
        ConnConfig {
            port: 3306,
            user: None,
            host: None,
            password: None,
            database: None,
            unix_socket: None,
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_user(&self) -> Option<&CStr> {
        self.user.as_ref().map(|x| &**x)
    }

    pub fn get_host(&self) -> Option<&CStr> {
        self.host.as_ref().map(|x| &**x)
    }

    pub fn get_password(&self) -> Option<&CStr> {
        self.password.as_ref().map(|x| &**x)
    }

    pub fn get_database(&self) -> Option<&CStr> {
        self.database.as_ref().map(|x| &**x)
    }

    pub fn get_unix_socket(&self) -> Option<&CStr> {
        self.unix_socket.as_ref().map(|x| &**x)
    }

    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        return self;
    }

    pub fn set_user(&mut self, user: &String) -> &mut Self {
        if user.is_empty() {
            return self;
        }
        if let Ok(val) = CString::new(user.as_bytes()) {
            self.user = Some(val);
        }
        return self;
    }

    pub fn set_host(&mut self, host: &String) -> &mut Self {
        if host.is_empty() {
            return self;
        }
        if let Ok(val) = CString::new(host.as_bytes()) {
            self.host = Some(val);
        }
        return self;
    }

    pub fn set_password(&mut self, password: &String) -> &mut Self {
        if password.is_empty() {
            return self;
        }
        if let Ok(val) = CString::new(password.as_bytes()) {
            self.password = Some(val);
        }
        return self;
    }

    pub fn set_database(&mut self, database: &String) -> &mut Self {
        if database.is_empty() {
            return self;
        }
        if let Ok(val) = CString::new(database.as_bytes()) {
            self.database = Some(val);
        }
        return self;
    }
}
