use crate::config::Config;
use mysqlclient_sys as ffi;
use std::cell::Cell;
use std::ffi::CStr;
use std::os::raw;
use std::ptr::{self, NonNull};
use std::sync::Once;

pub type MysqlRow = ffi::MYSQL_ROW;
pub type MysqlField = *mut ffi::MYSQL_FIELD;
pub type MysqlRes = *mut ffi::MYSQL_RES;

/// https://www.mysqlzh.com/api/66.html
pub struct MysqlConnect<'a> {
    config: &'a Config,
    mysql: NonNull<ffi::MYSQL>,
    mysql_res: Cell<MysqlRes>,
}

//用于运行一次性全局初始化
static MYSQL_SERVER_INIT: Once = Once::new();
static mut MYSQL_SERVER_INIT_RESULT: bool = false;

fn server_init() -> Result<(), String> {
    MYSQL_SERVER_INIT.call_once(|| unsafe {
        if 0 == ffi::mysql_server_init(0, ptr::null_mut(), ptr::null_mut()) {
            MYSQL_SERVER_INIT_RESULT = true;
        } else {
            MYSQL_SERVER_INIT_RESULT = false;
        }
    });
    unsafe {
        if MYSQL_SERVER_INIT_RESULT {
            return Ok(());
        }
        return Err("Unable to perform MySQL global initialization".into());
    }
}

fn init() -> Result<NonNull<ffi::MYSQL>, String> {
    unsafe {
        let res = ffi::mysql_init(ptr::null_mut());
        if let Some(mysql) = NonNull::new(res) {
            return Ok(mysql);
        } else {
            return Err("Out of memory mysql_init".into());
        }
    }
}

impl<'a> Drop for MysqlConnect<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::mysql_close(self.mysql.as_ptr());
        }
    }
}

impl<'a> MysqlConnect<'a> {
    pub fn new(config: &'a Config) -> Result<Self, String> {
        server_init()?;
        let mysql = init()?;
        let mysql = MysqlConnect {
            mysql: mysql,
            config: config,
            mysql_res: Cell::new(ptr::null_mut()),
        };

        let charset_result = unsafe {
            ffi::mysql_options(
                mysql.mysql.as_ptr(),
                ffi::mysql_option::MYSQL_SET_CHARSET_NAME,
                b"utf8mb4\0".as_ptr() as *const raw::c_void,
            )
        };
        if charset_result != 0 {
            return Err("mysql set utf8mb4 error".into());
        }

        Ok(mysql)
    }

    pub fn connect(&self) -> Result<(), String> {
        let port = self.config.get_port();
        let user = self.config.get_user();
        let host = self.config.get_host();
        let password = self.config.get_password();
        let database = self.config.get_database();
        let unix_socket = self.config.get_unix_socket();

        let mysql = unsafe {
            // Make sure you don't use the fake one!
            ffi::mysql_real_connect(
                self.mysql.as_ptr(),
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

        if !mysql.is_null() {
            return Ok(());
        }
        return Err(self.last_error());
    }

    pub fn real_query(&self, sql: &String) -> Result<(), String> {
        unsafe {
            if !self.mysql_res.get().is_null() {
                ffi::mysql_free_result(self.mysql_res.get());
            }
            let res = ffi::mysql_real_query(
                self.mysql.as_ptr(),
                sql.as_ptr() as *const raw::c_char,
                sql.len() as raw::c_ulong,
            );
            if res == 0 {
                if let Ok(res) = self.store_result() {
                    if !res.is_null() {
                        self.mysql_res.set(res);
                    }
                    return Ok(());
                }
            }
            if let Err(err) = self.connect() {
                return Err(err);
            }
            let res = ffi::mysql_real_query(
                self.mysql.as_ptr(),
                sql.as_ptr() as *const raw::c_char,
                sql.len() as raw::c_ulong,
            );
            if res == 0 {
                match self.store_result() {
                    Ok(res) => {
                        if !res.is_null() {
                            self.mysql_res.set(res);
                        }
                        return Ok(());
                    }
                    Err(err) => return Err(err),
                }
            }
            return Err(self.last_error());
        }
    }

    #[inline]
    pub fn insert_id(&self) -> u64 {
        unsafe { ffi::mysql_insert_id(self.mysql.as_ptr()) as u64 }
    }

    #[inline]
    pub fn fetch_row(&self) -> MysqlRow {
        unsafe { ffi::mysql_fetch_row(self.mysql_res.get()) }
    }
    #[inline]
    /// 结果集的列  field.name
    pub fn fetch_field(&self) -> MysqlField {
        unsafe { ffi::mysql_fetch_field(self.mysql_res.get()) }
    }

    #[inline]
    /// 结果集的列数组  field[0].name
    pub fn fetch_fields(&self) -> MysqlField {
        unsafe { ffi::mysql_fetch_fields(self.mysql_res.get()) }
    }

    #[inline]
    /// 字段的数量
    pub fn num_rows(&self) -> u64 {
        unsafe { ffi::mysql_num_rows(self.mysql_res.get()) as u64 }
    }
    /// 结果的字段数量
    pub fn num_fields(&self) -> u32 {
        unsafe { ffi::mysql_num_fields(self.mysql_res.get()) as u32 }
    }

    #[inline]
    /// 字段值的长度。
    pub fn fetch_lengths(&self) -> u64 {
        unsafe { ffi::mysql_fetch_lengths(self.mysql_res.get()) as u64 }
    }

    #[inline]
    /// (UPDATE,DELETE,INSERT)语句影响的行数
    pub fn affected_rows(&self) -> u64 {
        unsafe { ffi::mysql_affected_rows(self.mysql.as_ptr()) as u64 }
    }

    /// 使SQL字符串合法
    pub fn real_escape_string(&self, sql_str: &String) -> String {
        let res = vec![0u8; sql_str.len() * 2 + 1];
        let res_len = unsafe {
            ffi::mysql_real_escape_string(
                self.mysql.as_ptr(),
                res.as_ptr() as *mut raw::c_char,
                sql_str.as_ptr() as *const raw::c_char,
                sql_str.len() as raw::c_ulong,
            )
        };
        String::from_utf8_lossy(&res[0..res_len as usize]).into_owned()
    }

    #[inline]
    /// 检查是否链接正常
    pub fn ping(&self) -> bool {
        unsafe { ffi::mysql_ping(self.mysql.as_ptr()) == 0 }
    }

    /// 设置mysql选项
    pub fn set_mysql_options(&self) -> Result<(), String> {
        self.real_query(&"SET sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT'))".into())?;
        self.real_query(&"SET time_zone = '+00:00';".into())?;
        self.real_query(&"SET character_set_client = 'utf8mb4'".into())?;
        self.real_query(&"SET character_set_connection = 'utf8mb4'".into())?;
        self.real_query(&"SET character_set_results = 'utf8mb4'".into())
    }

    #[inline]
    fn store_result(&self) -> Result<MysqlRes, String> {
        unsafe {
            let res = ffi::mysql_store_result(self.mysql.as_ptr());
            if !res.is_null() {
                return Ok(res);
            }
            if ffi::mysql_field_count(self.mysql.as_ptr()) == 0 {
                return Ok(res);
            }
            return Err(self.last_error());
        }
    }

    fn last_error(&self) -> String {
        unsafe { CStr::from_ptr(ffi::mysql_error(self.mysql.as_ptr())) }
            .to_string_lossy()
            .into_owned()
    }

    /*
    fn more_results(&self) -> bool {
        unsafe { ffi::mysql_more_results(self.mysql.as_ptr()) != 0 }
    }

    fn next_result(&self) -> Result<bool, String> {
        let res = unsafe { ffi::mysql_next_result(self.mysql.as_ptr()) };
        match res {
            0 => Ok(true),
            -1 => Ok(false),
            _ => Err(self.last_error()),
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

        /// batch_execute(|| { self.execute(sql) })
        pub fn batch_execute<T, F>(&self, callback: F) -> Result<T, String>
        where
            F: FnOnce() -> Result<T, String>,
        {
            unsafe {
                ffi::mysql_set_server_option(
                    self.mysql.as_ptr(),
                    ffi::enum_mysql_set_option::MYSQL_OPTION_MULTI_STATEMENTS_ON,
                );
            }

            self.last_error()?;
            let result = callback();

            unsafe {
                ffi::mysql_set_server_option(
                    self.mysql.as_ptr(),
                    ffi::enum_mysql_set_option::MYSQL_OPTION_MULTI_STATEMENTS_OFF,
                );
            }
            self.last_error()?;

            result
        }

        fn mysql_free_result(&self) -> Result<(), String> {
            unsafe {
                let res = ffi::mysql_store_result(self.mysql.as_ptr());
                if !res.is_null() {
                    ffi::mysql_free_result(res)
                }
            };
            self.last_error()
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
        */
}
