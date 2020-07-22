use std::ffi::{CStr, CString};

pub struct Config {
    port: u16,
    user: Option<CString>,
    host: Option<CString>,
    password: Option<CString>,
    database: Option<CString>,
    unix_socket: Option<CString>,
}

impl Config {
    pub fn new() -> Self {
        Config {
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

    pub fn set_unix_socket(&mut self, unix_socket: &String) -> &mut Self {
        if unix_socket.is_empty() {
            return self;
        }
        if let Ok(val) = CString::new(unix_socket.as_bytes()) {
            self.unix_socket = Some(val);
        }
        return self;
    }
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
