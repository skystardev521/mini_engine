use crate::config::ConnConfig;
use crate::qresult::MysqlResult;
use crate::qresult::QueryResult;
use mysqlclient_sys as ffi;
use std::ffi::CStr;
use std::os::raw;
use std::ptr::{self, NonNull};
//use std::sync::Once;

//www.mysqlzh.com/api/66.html
/// 查询过程中丢失了与MySQL服务器的连接
pub const CR_SERVER_LOST: u32 = 2013;

/// 连接到MySQL服务器失败
pub const CR_CONN_HOST_ERROR: u32 = 2003;

/// MySQL服务器不可用
pub const CR_SERVER_GONE_ERROR: u32 = 2006;

/*
//用于运行一次性全局初始化
//同一个变量 call_once 回调函数只运行一次
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
*/

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

pub struct Connect {
    config: ConnConfig,
    mysql: NonNull<ffi::MYSQL>,
}

impl Drop for Connect {
    fn drop(&mut self) {
        unsafe {
            ffi::mysql_close(self.mysql.as_ptr());
        }
    }
}

impl Connect {
    pub fn new(config: ConnConfig) -> Result<Self, String> {
        //server_init()?;
        let mysql = init()?;
        let mysql = Connect {
            mysql: mysql,
            config: config,
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
            return self.set_mysql_options();
        } else {
            return Err(self.last_error());
        }
    }
    /// UPDATE,DELETE,INSERT return affected_rows
    pub fn alter_data(&self, sql: &String) -> Result<u64, String> {
        let res = unsafe {
            ffi::mysql_real_query(
                self.mysql.as_ptr(),
                sql.as_ptr() as *const raw::c_char,
                sql.len() as raw::c_ulong,
            )
        };

        if res == 0 {
            return Ok(self.affected_rows());
        }

        self.check_connect()?;

        let res = unsafe {
            ffi::mysql_real_query(
                self.mysql.as_ptr(),
                sql.as_ptr() as *const raw::c_char,
                sql.len() as raw::c_ulong,
            )
        };
        if res == 0 {
            return Ok(self.affected_rows());
        }
        return Err(self.last_error());
    }

    pub fn query_data(&self, sql: &String) -> Result<QueryResult<MysqlResult>, String> {
        let res = unsafe {
            ffi::mysql_real_query(
                self.mysql.as_ptr(),
                sql.as_ptr() as *const raw::c_char,
                sql.len() as raw::c_ulong,
            )
        };
        if res == 0 {
            if let Ok(mysql_res) = self.store_result() {
                return Ok(QueryResult::new(mysql_res));
            }
        }

        self.check_connect()?;

        let res = unsafe {
            ffi::mysql_real_query(
                self.mysql.as_ptr(),
                sql.as_ptr() as *const raw::c_char,
                sql.len() as raw::c_ulong,
            )
        };
        if res == 0 {
            match self.store_result() {
                Ok(mysql_res) => {
                    return Ok(QueryResult::new(mysql_res));
                }
                Err(err) => return Err(err),
            }
        }
        return Err(self.last_error());
    }
    #[inline]
    #[allow(dead_code)]
    pub fn insert_id(&self) -> u64 {
        unsafe { ffi::mysql_insert_id(self.mysql.as_ptr()) as u64 }
    }

    #[inline]
    /// (UPDATE,DELETE,INSERT)语句影响的行数
    pub fn affected_rows(&self) -> u64 {
        unsafe { ffi::mysql_affected_rows(self.mysql.as_ptr()) as u64 }
    }

    #[inline]
    #[allow(dead_code)]
    /// 使 二进制或字符串 转成 合法SQL字符串
    pub fn real_escape_string(&self, vec_data: &[u8]) -> String {
        let res = vec![0u8; vec_data.len() * 2 + 1];
        let res_len = unsafe {
            ffi::mysql_real_escape_string(
                self.mysql.as_ptr(),
                res.as_ptr() as *mut raw::c_char,
                vec_data.as_ptr() as *const raw::c_char,
                vec_data.len() as raw::c_ulong,
            )
        };
        String::from_utf8_lossy(&res[0..res_len as usize]).into_owned()
    }

    #[inline]
    /// 检查是否链接正常
    pub fn ping(&self) {
        let res = unsafe { ffi::mysql_ping(self.mysql.as_ptr()) };
        if res == 0 {
            return;
        }
        if let Ok(()) = self.check_connect() {};
    }

    fn check_connect(&self) -> Result<(), String> {
        let errno = unsafe { ffi::mysql_errno(self.mysql.as_ptr()) };
        if errno == CR_SERVER_LOST || errno == CR_CONN_HOST_ERROR || errno == CR_SERVER_GONE_ERROR {
            self.connect()?;
        }
        Err(self.last_error())
    }

    /// 设置mysql选项
    pub fn set_mysql_options(&self) -> Result<(), String> {
        self.alter_data(&"SET sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT'))".into())?;
        self.alter_data(&"SET time_zone = '+00:00';".into())?;
        self.alter_data(&"SET character_set_client = 'utf8mb4'".into())?;
        self.alter_data(&"SET character_set_connection = 'utf8mb4'".into())?;
        self.alter_data(&"SET character_set_results = 'utf8mb4'".into())?;
        Ok(())
    }

    #[inline]
    fn store_result(&self) -> Result<*mut ffi::MYSQL_RES, String> {
        unsafe {
            let res = ffi::mysql_store_result(self.mysql.as_ptr());
            if !res.is_null() {
                return Ok(res);
            }
            return Err(self.last_error());
        }
    }

    fn last_error(&self) -> String {
        unsafe { CStr::from_ptr(ffi::mysql_error(self.mysql.as_ptr())) }
            .to_string_lossy()
            .to_string()
    }
}
