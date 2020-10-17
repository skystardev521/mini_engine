//mod ffi_wrapper;
//use ffi_wrapper::{add_w, add_v2_w, replace_w};

use std::ffi::{CStr, CString};
use std::os::raw::{c_char,c_void};

mod hiredis;

use hiredis::{freeReplyObject, redisFree};
use hiredis::{redisContext, redisReply, timeval};
use hiredis::{redisCommand, redisConnect, redisConnectWithTimeout};
use hiredis::{REDIS_OK, REDIS_REPLY_STATUS , REDIS_REPLY_NIL, REDIS_REPLY_INTEGER, REDIS_REPLY_STRING ,REDIS_REPLY_ARRAY,REDIS_REPLY_ERROR};

#[macro_export]
macro_rules! pr {
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

struct RedisClient {
    ctx: *mut redisContext,
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe {
                redisFree(self.ctx);
            }
        }
    }
}

impl RedisClient {
    pub fn redis_connect(ip: String, port: i32) -> Result<RedisClient, String> {
        match CString::new(ip) {
            Ok(cs) => {
                let ret_ptr = unsafe { redisConnect(cs.as_ptr(), port) };
                if ret_ptr.is_null() {
                    return Err("null ptr".into());
                }
                if unsafe { *ret_ptr }.err as u32 != REDIS_OK {
                    return Err(unsafe {
                        CStr::from_ptr((*ret_ptr).errstr.as_ptr())
                            .to_string_lossy()
                            .to_string()
                    });
                }
                return Ok(RedisClient { ctx: ret_ptr });
            }
            Err(nul_err) => {
                return Err(format!(
                    "redis ip to CString Error:{}",
                    nul_err.to_string()
                ))
            }
        }
    }

    /// timeout ms
    fn redis_connect_timeout(
        ip: String,
        port: i32,
        timeout: u32,
    ) -> Result<RedisClient, String> {
        match CString::new(ip) {
            Ok(cs) => {
                let tv_sec = (timeout / 1000) as i64;
                let tv_usec = ((timeout % 1000) * 1000) as i64;
                let tv = timeval {
                    tv_sec,
                    tv_usec
                };
                let ret_ptr = unsafe { redisConnectWithTimeout(cs.as_ptr(), port, tv) };
                if ret_ptr.is_null() {
                    return Err("null ptr".into());
                }
                if unsafe { *ret_ptr }.err as u32 != REDIS_OK {
                    return Err(unsafe {
                        CStr::from_ptr((*ret_ptr).errstr.as_ptr())
                            .to_string_lossy()
                            .to_string()
                    });
                }
                return Ok(RedisClient { ctx: ret_ptr });
            }
            Err(nul_err) => {
                return Err(format!(
                    "redis ip to CString Error:{}",
                    nul_err.to_string()
                ))
            }
        }
    }

    pub fn redis_command(&self, cmd: String) -> Result<*mut redisReply, String> {
        match CString::new(cmd) {
            Ok(cs) =>{ 
                unsafe {
                    let ret = redisCommand(self.ctx, cs.as_ptr());
                    if ret.is_null() {
                        return Err("result is null".into());
                    }
                    let reply = ret as *mut redisReply;
                    if (*reply).type_ as u32 == REDIS_REPLY_ERROR {
                        if (*reply).str_.is_null() {
                            freeReplyObject(reply as *mut c_void);
                            return Err("REDIS_REPLY_ERROR NULL".into());
                        }
                        freeReplyObject(reply as *mut c_void);
                        return Err(CStr::from_ptr((*reply).str_).to_string_lossy().to_string());
                    }
                    return Ok(reply);
                }
            },
            Err(nul_err) => {
                return Err(format!(
                    "redis cmd to CString Error:{}",
                    nul_err.to_string()
                ));
            }
        }
    }

    
    pub fn redis_cmd_i64(&self, cmd: String)->Result<i64, String>{
        match self.redis_command(cmd){
            Ok(reply)=>{
                unsafe{
                    if (*reply).type_ as u32 == REDIS_REPLY_NIL {
                        freeReplyObject(reply as *mut c_void);
                        return Err("REDIS_REPLY_NIL".into());
                    }
                    if (*reply).type_ as u32 != REDIS_REPLY_INTEGER{
                        freeReplyObject(reply as *mut c_void);
                        return Err("reply type error".into())
                    }
                    let value = (*reply).integer;
                    freeReplyObject(reply as *mut c_void);
                    return Ok(value)
                }
            }
            Err(err)=> return Err(err)
        }
    }

    pub fn redis_cmd_ok(&self, cmd: String)->Result<bool, String>{
        match self.redis_command(cmd){
            Ok(reply)=>{
                unsafe{
                    if (*reply).type_ as u32 == REDIS_REPLY_NIL {
                        freeReplyObject(reply as *mut c_void);
                        return Err("REDIS_REPLY_NIL".into());
                    }
                    if (*reply).type_ as u32 != REDIS_REPLY_STATUS {
                        freeReplyObject(reply as *mut c_void);
                        return Err("reply type error".into());
                    }
                    if CStr::from_ptr((*reply).str_).to_bytes() == b"OK"{
                        freeReplyObject(reply as *mut c_void);
                        return Ok(true);
                    }
                    let reason = CStr::from_ptr((*reply).str_).to_string_lossy().to_string();
                    freeReplyObject(reply as *mut c_void);
                    return Err(reason);
                }
            }
            Err(err)=> return Err(err)
        }
    }

    pub fn redis_cmd_str(&self, cmd: String)->Result<String, String>{
        match self.redis_command(cmd){
            Ok(reply)=>{
                unsafe{
                    if (*reply).type_ as u32 == REDIS_REPLY_NIL {
                        freeReplyObject(reply as *mut c_void);
                        return Err("REDIS_REPLY_NIL".into());
                    }
                    if (*reply).type_ as u32 != REDIS_REPLY_STRING {
                        freeReplyObject(reply as *mut c_void);
                        return Err("reply type error".into());
                    }
                    let result = CStr::from_ptr((*reply).str_).to_string_lossy().to_string();
                    freeReplyObject(reply as *mut c_void);
                    return Ok(result);
                }
            }
            Err(err)=> return Err(err)
        }
    }

    
    pub fn redis_cmd_vec_i64(&self, cmd: String)->Result<Vec<i64>, String>{
        match self.redis_command(cmd){
            Ok(reply)=>{
                unsafe{
                    if (*reply).type_ as u32 == REDIS_REPLY_NIL {
                        freeReplyObject(reply as *mut c_void);
                        return Err("REDIS_REPLY_NIL".into());
                    }
                    if (*reply).type_ as u32 != REDIS_REPLY_ARRAY {
                        freeReplyObject(reply as *mut c_void);
                        return Err("reply type error".into());
                    }
                    let mut vec_i64 = Vec::new();
                    let elements = (*reply).elements as usize;
                    let reply_slice = std::ptr::slice_from_raw_parts((*reply).element, elements);
                    for i in 0 .. elements{
                        let e_reply = (&*reply_slice)[i];
                        if (*e_reply).type_ as u32 == REDIS_REPLY_INTEGER{
                            vec_i64.push((*e_reply).integer);
                        }
                        freeReplyObject(e_reply as *mut c_void);
                    }
                    freeReplyObject(reply as *mut c_void);
                    return Ok(vec_i64);
                }
            }
            Err(err)=> return Err(err)
        }
    }

    pub fn redis_cmd_vec_str(&self, cmd: String)->Result<Vec<String>, String>{
        match self.redis_command(cmd){
            Ok(reply)=>{
                unsafe{
                    if (*reply).type_ as u32 == REDIS_REPLY_NIL {
                        freeReplyObject(reply as *mut c_void);
                        return Err("REDIS_REPLY_NIL".into());
                    }
                    if (*reply).type_ as u32 != REDIS_REPLY_ARRAY {
                        freeReplyObject(reply as *mut c_void);
                        return Err("reply type error".into());
                    }
                    let mut vec_str = Vec::new();
                    let elements = (*reply).elements as usize;
                    let reply_slice = std::ptr::slice_from_raw_parts((*reply).element, elements);
                    for i in 0 .. elements{
                        let e_reply = (&*reply_slice)[i];
                        if (*e_reply).type_ as u32 == REDIS_REPLY_STRING{
                            vec_str.push(CStr::from_ptr((*e_reply).str_).to_string_lossy().to_string());
                        }
                        freeReplyObject(e_reply as *mut c_void);
                    }
                    freeReplyObject(reply as *mut c_void);
                    return Ok(vec_str);
                }
            }
            Err(err)=> return Err(err)
        }
    }
    

}



pub fn timestamp() -> u64 {
    let now = std::time::SystemTime::now();
    match now.duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

fn main() {
    pr!("pr:{}", "xx");

    match RedisClient::redis_connect_timeout(String::from("127.0.0.1"), 6379, 1000) {
        Ok(client) => {
            let mut n = 0;
            let s_ts = timestamp();
            println!("start ts:{}", s_ts);
            loop {
                n += 1;
                if n == 2 {
                    let e_ts = timestamp();
                    println!("end ts:{} total:{}", e_ts, e_ts - s_ts);
                    break;
                }
                let cmd = format!("zrevrange test_2017 0 -1 withscores");
                //let cmd = format!("hmset hkey k1 12345 k2 12345678");
                
                match client.redis_cmd_vec_str(cmd) {
                //match client.redis_cmd_ok(cmd) {
                    Ok(reply_ptr) => {
                        println!("reply_ptr:{}", reply_ptr.len())
                        /*
                        let cs = unsafe { CStr::from_ptr((*reply_ptr).str_) };
                        println!(
                            "cmd:{}, reply:{}",
                            "set key_ok val_ok",
                            cs.to_string_lossy().to_string()
                        );
                        */
                    }
                    Err(err) => println!("redis_command err:{}", err),
                }
            }
        }
        Err(err) => println!("connect err:{}", err),
    }

    /*
    println!("{}+ {} = {}", 3, 5, add_w(3, 5));

    let mut s: [u8;3] = [0u8;3];

    let mut d: [u8;3] = [0u8;3];

    println!("s:{:?}\nd:{:?}", s, d);

    replace_w(&mut s,&mut d);

    println!("s:{:?}\nd:{:?}", s, d);



    let ref mut sum: i32 = 0;

    add_v2_w(3, 5, sum);

    println!("{}+ {} = {}", 3, 5, sum);
    */
}
