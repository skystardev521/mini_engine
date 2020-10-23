#![allow(dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals)]


pub const REDIS_ERR: i32 = -1;

pub const REDIS_OK: u32 = 0;

pub const REDIS_ERR_IO: u32 = 1;

pub const REDIS_ERR_EOF: u32 = 3;

pub const REDIS_ERR_PROTOCOL: u32 = 4;

pub const REDIS_ERR_OOM: u32 = 5;

pub const REDIS_ERR_TIMEOUT: u32 = 6;

pub const REDIS_ERR_OTHER: u32 = 2;

pub const REDIS_REPLY_STRING: u32 = 1;

pub const REDIS_REPLY_ARRAY: u32 = 2;

pub const REDIS_REPLY_INTEGER: u32 = 3;

pub const REDIS_REPLY_NIL: u32 = 4;

pub const REDIS_REPLY_STATUS: u32 = 5;

pub const REDIS_REPLY_ERROR: u32 = 6;

pub const REDIS_REPLY_DOUBLE: u32 = 7;

pub const REDIS_REPLY_BOOL: u32 = 8;

pub const REDIS_REPLY_MAP: u32 = 9;

pub const REDIS_REPLY_SET: u32 = 10;

pub const REDIS_REPLY_ATTR: u32 = 11;

pub const REDIS_REPLY_PUSH: u32 = 12;

pub const REDIS_REPLY_BIGNUM: u32 = 13;

pub const REDIS_REPLY_VERB: u32 = 14;

#[allow(non_camel_case_types)]
pub type redisFD = ::std::os::raw::c_int;
#[allow(non_camel_case_types)]
pub type size_t = ::std::os::raw::c_ulong;
#[allow(non_camel_case_types)]
pub type ssize_t = ::std::os::raw::c_long;
#[allow(non_camel_case_types)]
pub type redisConnectionType = ::std::os::raw::c_uint;
#[allow(non_camel_case_types)]
pub type __time_t = ::std::os::raw::c_long;
#[allow(non_camel_case_types)]
pub type __suseconds_t = ::std::os::raw::c_long;

extern "C" {

    pub fn redisFree(c: *mut redisContext);

    pub fn freeReplyObject(reply: *mut ::std::os::raw::c_void);

    pub fn redisGetReply(
        c: *mut redisContext,
        reply: *mut *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;

    pub fn redisConnect(
        ip: *const ::std::os::raw::c_char,
        port: ::std::os::raw::c_int,
    ) -> *mut redisContext;

    pub fn redisConnectWithTimeout(
        ip: *const ::std::os::raw::c_char,
        port: ::std::os::raw::c_int,
        tv: timeval,
    ) -> *mut redisContext;

    pub fn redisConnectUnix(path: *const ::std::os::raw::c_char) -> *mut redisContext;

    pub fn redisConnectUnixWithTimeout(
        path: *const ::std::os::raw::c_char,
        tv: timeval,
    ) -> *mut redisContext;

    pub fn redisReconnect(c: *mut redisContext) -> ::std::os::raw::c_int;

    pub fn redisSetTimeout(c: *mut redisContext, tv: timeval) -> ::std::os::raw::c_int;

    pub fn redisEnableKeepAlive(c: *mut redisContext) -> ::std::os::raw::c_int;

    pub fn redisCommand(
        c: *mut redisContext,
        format: *const ::std::os::raw::c_char,
        ...
    ) -> *mut ::std::os::raw::c_void;

    pub fn redisAppendCommand(
        c: *mut redisContext,
        format: *const ::std::os::raw::c_char,
        ...
    ) -> ::std::os::raw::c_int;

}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct redisReply {
    pub type_: ::std::os::raw::c_int,
    pub integer: ::std::os::raw::c_longlong,
    pub dval: f64,
    pub len: size_t,
    pub str_: *mut ::std::os::raw::c_char,
    pub vtype: [::std::os::raw::c_char; 4usize],
    pub elements: size_t,
    pub element: *mut *mut redisReply,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct redisReplyObjectFunctions {
    pub createString: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *const redisReadTask,
            arg2: *mut ::std::os::raw::c_char,
            arg3: size_t,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub createArray: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *const redisReadTask,
            arg2: size_t,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub createInteger: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *const redisReadTask,
            arg2: ::std::os::raw::c_longlong,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub createDouble: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *const redisReadTask,
            arg2: f64,
            arg3: *mut ::std::os::raw::c_char,
            arg4: size_t,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub createNil: ::std::option::Option<
        unsafe extern "C" fn(arg1: *const redisReadTask) -> *mut ::std::os::raw::c_void,
    >,
    pub createBool: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *const redisReadTask,
            arg2: ::std::os::raw::c_int,
        ) -> *mut ::std::os::raw::c_void,
    >,

    pub freeObject: ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct redisReadTask {
    pub type_: ::std::os::raw::c_int,
    pub elements: ::std::os::raw::c_longlong,
    pub idx: ::std::os::raw::c_int,
    pub obj: *mut ::std::os::raw::c_void,
    pub parent: *mut redisReadTask,
    pub privdata: *mut ::std::os::raw::c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct redisReader {
    pub err: ::std::os::raw::c_int,
    pub errstr: [::std::os::raw::c_char; 128usize],
    pub buf: *mut ::std::os::raw::c_char,
    pub pos: size_t,
    pub len: size_t,
    pub maxbuf: size_t,
    pub maxelements: ::std::os::raw::c_longlong,
    pub task: *mut *mut redisReadTask,
    pub tasks: ::std::os::raw::c_int,
    pub ridx: ::std::os::raw::c_int,
    pub reply: *mut ::std::os::raw::c_void,
    pub fn_: *mut redisReplyObjectFunctions,
    pub privdata: *mut ::std::os::raw::c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr {
    pub _address: u8,
}

#[allow(non_camel_case_types)]
pub type redisPushFn = ::std::option::Option<
    unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void, arg2: *mut ::std::os::raw::c_void),
>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct redisContext {
    pub funcs: *const redisContextFuncs,
    pub err: ::std::os::raw::c_int,
    pub errstr: [::std::os::raw::c_char; 128usize],
    pub fd: redisFD,
    pub flags: ::std::os::raw::c_int,
    pub obuf: *mut ::std::os::raw::c_char,
    pub reader: *mut redisReader,
    pub connection_type: redisConnectionType,
    pub connect_timeout: *mut timeval,
    pub command_timeout: *mut timeval,
    pub tcp: redisContext__bindgen_ty_1,
    pub unix_sock: redisContext__bindgen_ty_2,
    pub saddr: *mut sockaddr,
    pub addrlen: size_t,
    pub privdata: *mut ::std::os::raw::c_void,
    pub free_privdata:
        ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>,
    pub privctx: *mut ::std::os::raw::c_void,
    pub push_cb: redisPushFn,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct redisAsyncContext {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct redisContextFuncs {
    pub free_privctx:
        ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>,
    pub async_read: ::std::option::Option<unsafe extern "C" fn(arg1: *mut redisAsyncContext)>,
    pub async_write: ::std::option::Option<unsafe extern "C" fn(arg1: *mut redisAsyncContext)>,
    pub read: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut redisContext,
            arg2: *mut ::std::os::raw::c_char,
            arg3: size_t,
        ) -> ssize_t,
    >,
    pub write: ::std::option::Option<unsafe extern "C" fn(arg1: *mut redisContext) -> ssize_t>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct redisContext__bindgen_ty_1 {
    pub host: *mut ::std::os::raw::c_char,
    pub source_addr: *mut ::std::os::raw::c_char,
    pub port: ::std::os::raw::c_int,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct redisContext__bindgen_ty_2 {
    pub path: *mut ::std::os::raw::c_char,
}
