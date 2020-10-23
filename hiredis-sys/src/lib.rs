#![allow(dead_code)]

mod bindings;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::slice_from_raw_parts;

use crate::bindings::redisCommand;
use crate::bindings::{freeReplyObject, redisFree};
use crate::bindings::{
    redisConnect, redisConnectUnix, redisConnectUnixWithTimeout, redisConnectWithTimeout,
    redisReconnect,
};
use crate::bindings::{redisContext, redisReply, timeval};

use crate::bindings::{
    REDIS_ERR_EOF, REDIS_ERR_IO, REDIS_OK, REDIS_REPLY_ARRAY, REDIS_REPLY_ERROR,
    REDIS_REPLY_INTEGER, REDIS_REPLY_NIL, REDIS_REPLY_STATUS, REDIS_REPLY_STRING,
};

#[macro_export]
macro_rules! test_fmt {
    ($fmt:expr, $($arg:expr),+ ) => {
        println!($fmt,
            $(
                {
                    $arg
                },
            )+
        );
    };
}

pub struct RedisClient {
    ctx_ptr: *mut redisContext,
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        if !self.ctx_ptr.is_null() {
            unsafe {
                redisFree(self.ctx_ptr);
            }
        }
    }
}

impl RedisClient {
    #[inline]
    fn ms_to_tv(ms: i64) -> timeval {
        let tv_sec = (ms / 1000) as i64;
        let tv_usec = ((ms % 1000) * 1000) as i64;
        timeval { tv_sec, tv_usec }
    }

    #[inline]
    fn c_char_ptr_to_string(c_char_ptr: *const c_char) -> String {
        unsafe { CStr::from_ptr(c_char_ptr).to_string_lossy().to_string() }
    }
    /// let em_reply_ptr: *mut T;
    /// Self::free_reply_object(em_reply_ptr as *mut _ as * mut c_void);
    #[inline]
    fn free_reply_object(reply_ptr: *mut c_void) {
        if reply_ptr.is_null() {
            return;
        }
        unsafe { freeReplyObject(reply_ptr) }
    }
    #[inline]
    fn get_client_status(ctx_ptr: *mut redisContext) -> Option<String> {
        let ctx_st = unsafe { &*ctx_ptr };
        if ctx_st.err as u32 == REDIS_OK {
            return None;
        }
        if ctx_st.err as u32 == REDIS_ERR_IO || ctx_st.err as u32 == REDIS_ERR_EOF {
            if unsafe { redisReconnect(ctx_ptr) } as u32 == REDIS_OK {
                return None;
            }
        }
        let c_ptr = ctx_st.errstr.as_ptr();
        return Some(Self::c_char_ptr_to_string(c_ptr));
    }

    pub fn redis_connect(ip: String, port: i32) -> Result<RedisClient, String> {
        match CString::new(ip) {
            Ok(c_ip) => {
                let ctx_ptr = unsafe { redisConnect(c_ip.as_ptr(), port) };
                if ctx_ptr.is_null() {
                    return Err("redisConnect return null ptr".into());
                }
                if let Some(err) = Self::get_client_status(ctx_ptr) {
                    return Err(err);
                }
                return Ok(RedisClient { ctx_ptr });
            }
            Err(nul_err) => {
                return Err(format!("redis ip to CString Error:{}", nul_err.to_string()))
            }
        }
    }

    pub fn redis_connect_unix(path: String) -> Result<RedisClient, String> {
        match CString::new(path) {
            Ok(c_path) => {
                let ctx_ptr = unsafe { redisConnectUnix(c_path.as_ptr()) };
                if ctx_ptr.is_null() {
                    return Err("redisConnect return null ptr".into());
                }
                if let Some(err) = Self::get_client_status(ctx_ptr) {
                    return Err(err);
                }
                return Ok(RedisClient { ctx_ptr });
            }
            Err(nul_err) => {
                return Err(format!("redis ip to CString Error:{}", nul_err.to_string()))
            }
        }
    }

    /// timeout ms
    pub fn redis_connect_timeout(
        ip: String,
        port: i32,
        timeout: u32,
    ) -> Result<RedisClient, String> {
        match CString::new(ip) {
            Ok(c_ip) => {
                let tv = Self::ms_to_tv(timeout as i64);
                let ctx_ptr = unsafe { redisConnectWithTimeout(c_ip.as_ptr(), port, tv) };
                if ctx_ptr.is_null() {
                    return Err("redis_connect_timeout return null ptr".into());
                }
                if let Some(err) = Self::get_client_status(ctx_ptr) {
                    return Err(err);
                }
                return Ok(RedisClient { ctx_ptr });
            }
            Err(nul_err) => {
                return Err(format!("redis ip to CString Error:{}", nul_err.to_string()))
            }
        }
    }

    pub fn redis_connect_unix_timeout(path: String, timeout: u32) -> Result<RedisClient, String> {
        match CString::new(path) {
            Ok(c_path) => {
                let tv = Self::ms_to_tv(timeout as i64);
                let ctx_ptr = unsafe { redisConnectUnixWithTimeout(c_path.as_ptr(), tv) };
                if ctx_ptr.is_null() {
                    return Err("redis_connect_timeout return null ptr".into());
                }
                if let Some(err) = Self::get_client_status(ctx_ptr) {
                    return Err(err);
                }
                return Ok(RedisClient { ctx_ptr });
            }
            Err(nul_err) => {
                return Err(format!("redis ip to CString Error:{}", nul_err.to_string()))
            }
        }
    }

    pub fn redis_command(&self, cmd: String) -> Result<*mut c_void, String> {
        match CString::new(cmd) {
            Ok(c_cmd) => {
                let mut reply_ptr = unsafe { redisCommand(self.ctx_ptr, c_cmd.as_ptr()) };
                if let Some(err) = Self::get_client_status(self.ctx_ptr) {
                    return Err(err);
                }
                if reply_ptr.is_null() {
                    reply_ptr = unsafe { redisCommand(self.ctx_ptr, c_cmd.as_ptr()) };
                    if reply_ptr.is_null() {
                        return Err("redisCommand return null".into());
                    }
                }
                let reply_st = unsafe { &mut *(reply_ptr as *mut redisReply) };
                if reply_st.type_ as u32 == REDIS_REPLY_ERROR {
                    if reply_st.str_.is_null() {
                        Self::free_reply_object(reply_ptr);
                        return Err("REDIS_REPLY_ERROR NULL".into());
                    } else {
                        let reason = Self::c_char_ptr_to_string(reply_st.str_);
                        Self::free_reply_object(reply_ptr);
                        return Err(reason);
                    }
                }
                return Ok(reply_ptr);
            }
            Err(nul_err) => {
                return Err(format!(
                    "redis cmd to CString Error:{}",
                    nul_err.to_string()
                ));
            }
        }
    }

    pub fn redis_cmd_i64(&self, cmd: String) -> Result<i64, String> {
        match self.redis_command(cmd) {
            Ok(reply_ptr) => {
                let reply_st = unsafe { &mut *(reply_ptr as *mut redisReply) };
                if reply_st.type_ as u32 == REDIS_REPLY_NIL {
                    Self::free_reply_object(reply_ptr);
                    return Err("REDIS_REPLY_NIL".into());
                }
                if reply_st.type_ as u32 != REDIS_REPLY_INTEGER {
                    Self::free_reply_object(reply_ptr);
                    return Err("reply type error".into());
                }
                let value = reply_st.integer;
                Self::free_reply_object(reply_ptr);
                return Ok(value);
            }
            Err(err) => return Err(err),
        }
    }

    pub fn redis_cmd_ok(&self, cmd: String) -> Result<bool, String> {
        match self.redis_command(cmd) {
            Ok(reply_ptr) => {
                let reply_st = unsafe { &mut *(reply_ptr as *mut redisReply) };
                if reply_st.type_ as u32 == REDIS_REPLY_NIL {
                    Self::free_reply_object(reply_ptr);
                    return Err("REDIS_REPLY_NIL".into());
                }
                if reply_st.type_ as u32 != REDIS_REPLY_STATUS {
                    Self::free_reply_object(reply_ptr);
                    return Err("reply type error".into());
                }
                let str_ptr = unsafe { CStr::from_ptr(reply_st.str_) };
                if str_ptr.to_bytes() == b"OK" {
                    Self::free_reply_object(reply_ptr);
                    return Ok(true);
                }
                Self::free_reply_object(reply_ptr);
                return Err(str_ptr.to_string_lossy().to_string());
            }
            Err(err) => return Err(err),
        }
    }

    pub fn redis_cmd_str(&self, cmd: String) -> Result<String, String> {
        match self.redis_command(cmd) {
            Ok(reply_ptr) => {
                let reply_st = unsafe { &mut *(reply_ptr as *mut redisReply) };
                if reply_st.type_ as u32 == REDIS_REPLY_NIL {
                    Self::free_reply_object(reply_ptr);
                    return Err("REDIS_REPLY_NIL".into());
                }
                if reply_st.type_ as u32 != REDIS_REPLY_STRING {
                    Self::free_reply_object(reply_ptr);
                    return Err("reply type error".into());
                }
                let result = Self::c_char_ptr_to_string(reply_st.str_);
                Self::free_reply_object(reply_ptr);
                return Ok(result);
            }
            Err(err) => return Err(err),
        }
    }

    pub fn redis_cmd_vec_i64(&self, cmd: String) -> Result<Vec<i64>, String> {
        match self.redis_command(cmd) {
            Ok(reply_ptr) => {
                let reply_st = unsafe { &*(reply_ptr as *mut redisReply) };
                if reply_st.type_ as u32 == REDIS_REPLY_NIL {
                    Self::free_reply_object(reply_ptr);
                    return Err("REDIS_REPLY_NIL".into());
                }
                if reply_st.type_ as u32 != REDIS_REPLY_ARRAY {
                    Self::free_reply_object(reply_ptr);
                    return Err("reply type error".into());
                }
                let mut result_vec_i64 = Vec::new();
                let elements = reply_st.elements as usize;
                let slice_reply = slice_from_raw_parts(reply_st.element, elements);
                for i in 0..elements {
                    let em_reply_st = unsafe { &*(&*slice_reply)[i] };
                    if em_reply_st.type_ as u32 == REDIS_REPLY_NIL {
                        continue;
                    }
                    if em_reply_st.type_ as u32 == REDIS_REPLY_INTEGER {
                        result_vec_i64.push(em_reply_st.integer);
                    }
                }
                Self::free_reply_object(reply_ptr);
                return Ok(result_vec_i64);
            }
            Err(err) => return Err(err),
        }
    }

    pub fn redis_cmd_vec_str(&self, cmd: String) -> Result<Vec<String>, String> {
        match self.redis_command(cmd) {
            Ok(reply_ptr) => {
                let reply = unsafe { &*(reply_ptr as *mut redisReply) };
                if reply.type_ as u32 == REDIS_REPLY_NIL {
                    Self::free_reply_object(reply_ptr);
                    return Err("REDIS_REPLY_NIL".into());
                }
                if reply.type_ as u32 != REDIS_REPLY_ARRAY {
                    Self::free_reply_object(reply_ptr);
                    return Err("reply type error".into());
                }
                let mut result_vec_str = Vec::new();
                let elements = (*reply).elements as usize;
                let slice_reply = std::ptr::slice_from_raw_parts((*reply).element, elements);
                for i in 0..elements {
                    let em_reply_st = unsafe { &*(&*slice_reply)[i] };
                    if em_reply_st.type_ as u32 == REDIS_REPLY_NIL {
                        continue;
                    }
                    if em_reply_st.type_ as u32 == REDIS_REPLY_STRING {
                        result_vec_str.push(Self::c_char_ptr_to_string(em_reply_st.str_));
                    }
                }
                Self::free_reply_object(reply_ptr);
                return Ok(result_vec_str);
            }
            Err(err) => return Err(err),
        }
    }
}

#[test]
pub fn test_redis_client() {
    test_fmt!("test_fmt:{} {} {}", "a", "b", "c");

    match RedisClient::redis_connect_timeout(String::from("127.0.0.1"), 6379, 1000) {
        Ok(client) => {
            let mut n = 0;
            loop {
                n += 1;
                if n == 10 {
                    break;
                }
                let cmd = format!("HGETALL runoobkey");
                match client.redis_cmd_vec_str(cmd) {
                    Ok(_vec_str) => {}
                    Err(err) => println!("redis_command err:{}", err),
                }
            }
        }
        Err(err) => println!("connect err:{}", err),
    }
}
