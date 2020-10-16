//mod ffi_wrapper;
//use ffi_wrapper::{add_w, add_v2_w, replace_w};

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod hiredis;

use hiredis::{redisCommand, redisConnect, redisContext, redisReply, REDIS_OK, REDIS_REPLY_ERROR};

fn redis_connect(ip: &String, port: i32) -> Result<*mut redisContext, String> {
    unsafe {
        let ret = redisConnect(ip.as_ptr() as *const c_char, port);
        if ret.is_null() {
            return Err("null ptr".into());
        }
        if (*ret).err as u32 != REDIS_OK {
            let err_str = CStr::from_ptr((*ret).errstr.as_ptr());
            return Err(err_str.to_string_lossy().to_string());
        }
        return Ok(ret);
    }
}

fn rs_cs(rs: String) -> CString {
    CString::new(rs).unwrap()
}


#[macro_export]
macro_rules! test {
    ($fmt:expr, $($arg:expr),+ ) => {

        let x = ($(
            {
                $arg
            },
        )+);

        println!( x );

        println!($fmt,
            $(
                {
                    $arg
                },
            )+
        );
    };
}

fn redis_command(ctx: *mut redisContext, cmd: String) -> Result<*mut redisReply, String> {
    match CString::new(cmd) {
        Ok(cs) => unsafe {
            let ret = redisCommand(
                ctx,
                cs.as_ptr()
            );
            if ret.is_null() {
                return Err("result is null".into());
            }
            let reply = ret as *mut redisReply;
            if (*reply).type_ as u32 == REDIS_REPLY_ERROR {
                if (*reply).str_.is_null() {
                    return Err("REDIS_REPLY_ERROR NULL".into());
                }
                return Err(CStr::from_ptr((*reply).str_).to_string_lossy().to_string());
            }
            return Ok(reply);
        },
        Err(nul_err) => return Err(nul_err.to_string()),
    }
}

fn main() {
    //test!("a:{} {}", "b", "c");

    match redis_connect(&String::from("127.0.0.1"), 6379) {
        Ok(conn) => match redis_command(conn, String::from("get new_key")) {
            Ok(reply_ptr) => {
                let cs = unsafe{ CStr::from_ptr((*reply_ptr).str_) };
                println!("reply:{}", cs.to_string_lossy().to_string());
            }
            Err(err) => println!("redis_command err:{}", err),
        },
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
