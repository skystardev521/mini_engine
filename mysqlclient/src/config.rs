use std::ffi::{CStr, CString};
use std::time::Duration;
#[derive(Clone)]
pub struct ThreadConfig {
    name: String,
    thread_num: u8,
    stack_size: usize,
    channel_size: u16,
    receiver_max_num: u16,
    ping_duration: Duration,
    sleep_duration: Duration,
    //vec_conn_config: Vec<ConnConfig>,
}

impl ThreadConfig {
    pub fn new() -> Self {
        ThreadConfig {
            thread_num: 3,
            stack_size: 0,
            channel_size: 1024,
            receiver_max_num: 1024,
            name: String::from("mysql"),
            ping_duration: Duration::from_secs(300),
            sleep_duration: Duration::from_millis(1),
            //vec_conn_config: vec![],
        }
    }

    pub fn get_thread_num(&self) -> u8 {
        self.thread_num
    }

    pub fn get_stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn get_channel_size(&self) -> u16 {
        self.channel_size
    }

    pub fn get_receiver_max_num(&self) -> u16 {
        self.receiver_max_num
    }
    /// sec
    pub fn get_ping_duration(&self) -> Duration {
        self.ping_duration
    }

    /// millis
    pub fn get_sleep_duration(&self) -> Duration {
        self.sleep_duration
    }

    /*
    pub fn get_conn_config(&self) -> &Vec<ConnConfig> {
        &self.vec_conn_config
    }
    */

    pub fn set_thread_num(&mut self, num: u8) -> &mut Self {
        self.thread_num = if num == 0 { 1 } else { num };
        self
    }
    pub fn set_stack_size(&mut self, num: usize) -> &mut Self {
        self.stack_size = num;
        self
    }

    pub fn set_channel_size(&mut self, num: u16) -> &mut Self {
        self.channel_size = if num < 128 { 128 } else { num };
        self
    }

    pub fn set_receiver_max_num(&mut self, num: u16) -> &mut Self {
        self.receiver_max_num = if num < 128 { 128 } else { num };
        self
    }

    /// sec
    pub fn set_ping_duration(&mut self, num: u16) -> &mut Self {
        let n = if num == 0 { 1 } else { num } as u64;
        self.ping_duration = Duration::from_secs(n);
        self
    }

    /// millis
    pub fn set_sleep_duration(&mut self, num: u16) -> &mut Self {
        let n = if num == 0 { 1 } else { num } as u64;
        self.sleep_duration = Duration::from_millis(n);
        self
    }

    /*
    pub fn set_conn_config(&mut self, vec_conn_config: Vec<ConnConfig>) -> &mut Self {
        self.vec_conn_config = vec_conn_config;
        self
    }
    */
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

    /*
    pub fn set_unix_socket(&mut self, unix_socket: &String) -> &mut Self {
        if unix_socket.is_empty() {
            return self;
        }
        if let Ok(val) = CString::new(unix_socket.as_bytes()) {
            self.unix_socket = Some(val);
        }
        return self;
    }
    */
}
/*
fn decode_into_cstring(s: &str) -> ConnectionResult<CString> {
    let decoded = percent_decode(s.as_bytes())
        .decode_utf8()
        .map_err(|_| connection_url_error())?;
    CString::new(decoded.as_bytes()).map_err(Into::into)
}

fn connection_url_error() -> ConnectionError {
    let msg = "MySQL connection URLs must be in the form \
               `mysql://[[user]:[password]@]host[:port][/database][?unix_socket=socket-path]`";
    ConnectionError::InvalidConnectionUrl(msg.into())
}
*/
