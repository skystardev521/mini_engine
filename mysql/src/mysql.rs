use mysqlclient_sys as ffi;

use crate::config::Config;
use std::ffi::CStr;
use std::os::raw;
use std::ptr::{self, NonNull};
use std::sync::Once;

pub struct MysqlConnect(NonNull<ffi::MYSQL>);

//用于运行一次性全局初始化
static MYSQL_SERVER_INIT: Once = Once::new();
static mut MYSQL_SERVER_INIT_RESULT: bool = false;
fn mysql_server_init() -> Result<(), String> {
    MYSQL_SERVER_INIT.call_once(|| unsafe {
        if 0 == ffi::mysql_server_init(0, ptr::null_mut(), ptr::null_mut()) {
            MYSQL_SERVER_INIT_RESULT = true;
        }
        MYSQL_SERVER_INIT_RESULT = false;
    });
    unsafe {
        if MYSQL_SERVER_INIT_RESULT {
            return Ok(());
        }
        return Err("Unable to perform MySQL global initialization".into());
    }
}

fn mysql_init() -> Result<NonNull<ffi::MYSQL>, String> {
    unsafe {
        let res = ffi::mysql_init(ptr::null_mut());
        if let Some(mysql) = NonNull::new(res) {
            return Ok(mysql);
        } else {
            return Err("Out of memory mysql_init".into());
        }
    }
}

fn mysql_stmt_init() -> Result<NonNull<ffi::MYSQL_STMT>, String> {
    unsafe {
        let res = ffi::mysql_stmt_init(ptr::null_mut());
        if let Some(mysql_stmt) = NonNull::new(res) {
            return Ok(mysql_stmt);
        } else {
            return Err("Out of memory creating prepared statement".into());
        }
    }
}

impl Drop for MysqlConnect {
    fn drop(&mut self) {
        unsafe {
            ffi::mysql_close(self.0.as_ptr());
        }
    }
}

impl MysqlConnect {
    pub fn new() -> Result<Self, String> {
        mysql_server_init()?;
        let mysql = mysql_init()?;
        let mysql = MysqlConnect(mysql);
        let charset_result = unsafe {
            ffi::mysql_options(
                mysql.0.as_ptr(),
                ffi::mysql_option::MYSQL_SET_CHARSET_NAME,
                b"utf8mb4\0".as_ptr() as *const raw::c_void,
            )
        };
        if charset_result != 0 {
            return Err("mysql set utf8mb4 error".into());
        }

        Ok(mysql)
    }

    pub fn connect(&self, config: &Config) -> Result<(), String> {
        let port = config.get_port();
        let user = config.get_user();
        let host = config.get_host();
        let password = config.get_password();
        let database = config.get_database();
        let unix_socket = config.get_unix_socket();

        unsafe {
            // Make sure you don't use the fake one!
            ffi::mysql_real_connect(
                self.0.as_ptr(),
                host.map(CStr::as_ptr).unwrap_or_else(|| ptr::null_mut()),
                user.map(CStr::as_ptr).unwrap_or_else(|| ptr::null_mut()),
                password
                    .map(CStr::as_ptr)
                    .unwrap_or_else(|| ptr::null_mut()),
                database
                    .map(CStr::as_ptr)
                    .unwrap_or_else(|| ptr::null_mut()),
                u32::from(port),
                unix_socket
                    .map(CStr::as_ptr)
                    .unwrap_or_else(|| ptr::null_mut()),
                0,
            )
        };

        self.last_error()
    }

    pub fn execute(&self, query: &str) -> Result<(), String> {
        unsafe {
            // Make sure you don't use the fake one!
            ffi::mysql_real_query(
                self.0.as_ptr(),
                query.as_ptr() as *const raw::c_char,
                query.len() as raw::c_ulong,
            );
        }
        self.last_error()?;
        self.flush_pending_results()?;
        Ok(())
    }

    /// batch_execute(|| { self.execute(sql) })
    pub fn batch_execute<T, F>(&self, callback: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        unsafe {
            ffi::mysql_set_server_option(
                self.0.as_ptr(),
                ffi::enum_mysql_set_option::MYSQL_OPTION_MULTI_STATEMENTS_ON,
            );
        }

        self.last_error()?;
        let result = callback();

        unsafe {
            ffi::mysql_set_server_option(
                self.0.as_ptr(),
                ffi::enum_mysql_set_option::MYSQL_OPTION_MULTI_STATEMENTS_OFF,
            );
        }
        self.last_error()?;

        result
    }

    pub fn affected_rows(&self) -> usize {
        unsafe { ffi::mysql_affected_rows(self.0.as_ptr()) as usize }
    }

    /*
    pub fn prepare(&self, query: &str) -> Result<Statement, String> {
        let stmt = mysql_stmt_init()?;
        let stmt = Statement::new(stmt);
        stmt.prepare(query)?;
        Ok(stmt)
    }
    */

    pub fn set_mysql_options(&self) -> Result<(), String> {
        self.execute("SET sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT'))")?;
        self.execute("SET time_zone = '+00:00';")?;
        self.execute("SET character_set_client = 'utf8mb4'")?;
        self.execute("SET character_set_connection = 'utf8mb4'")?;
        self.execute("SET character_set_results = 'utf8mb4'")?;
        Ok(())
    }

    fn mysql_free_result(&self) -> Result<(), String> {
        unsafe {
            let res = ffi::mysql_store_result(self.0.as_ptr());
            if res.is_null() == false {
                ffi::mysql_free_result(res);
            }
        }
        self.last_error()
    }

    fn more_results(&self) -> bool {
        unsafe { ffi::mysql_more_results(self.0.as_ptr()) != 0 }
    }

    fn next_result(&self) -> Result<(), String> {
        unsafe { ffi::mysql_next_result(self.0.as_ptr()) };
        self.last_error()
    }

    fn last_error(&self) -> Result<(), String> {
        let last_error = unsafe { CStr::from_ptr(ffi::mysql_error(self.0.as_ptr())) }
            .to_string_lossy()
            .into_owned();

        if last_error.is_empty() {
            Ok(())
        } else {
            Err(last_error)
        }
    }

    fn flush_pending_results(&self) -> Result<(), String> {
        // We may have a result to process before advancing
        self.mysql_free_result()?;
        while self.more_results() {
            self.next_result()?;
            self.mysql_free_result()?;
        }
        Ok(())
    }
}
