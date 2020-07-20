use mysql::test;
use mysql::Config;
use mysql::Connect;
use mysql::QueryResult;

use std::ffi::CStr;
use std::ptr::{self, NonNull};

pub type TINYINT = i8;
pub type SMALLINT = i16;
pub type INTEGER = i32;
pub type BIGINT = i64;
pub type FLOAT = f32;
pub type DOUBLE = f64;
pub type BLOB = Vec<i8>;
pub type MyStr = String;

#[macro_use]


/*
trait Value<T> {
    fn data(&self, ptr: *const i8, size: usize) -> T;
}

impl Value<i8> for i8 {
    fn data(&self, ptr: *const i8, _size: usize) -> i8 {
        if ptr.is_null() {
            return *self;
        }
        let cs = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        if let Ok(val) = cs.parse::<i8>() {
            return val as i8;
        }
        return *self;
    }
}

impl Value<i16> for i16 {
    fn data(&self, ptr: *const i8, _size: usize) -> i16 {
        if ptr.is_null() {
            return *self;
        }
        let cs = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        if let Ok(val) = cs.parse::<i16>() {
            return val;
        }
        return *self;
    }
}

impl Value<i32> for i32 {
    fn data(&self, ptr: *const i8, _size: usize) -> i32 {
        if ptr.is_null() {
            return *self;
        }
        let cs = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        if let Ok(val) = cs.parse::<i32>() {
            return val;
        }
        return *self;
    }
}

impl Value<f64> for f64 {
    fn data(&self, ptr: *const i8, _size: usize) -> f64 {
        if ptr.is_null() {
            return *self;
        }
        let cs = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        if let Ok(val) = cs.parse::<f64>() {
            return val;
        }
        return *self;
    }
}

impl Value<Vec<i8>> for Vec<i8> {
    fn data(&self, ptr: *const i8, size: usize) -> Vec<i8> {
        if ptr.is_null() {
            return self.to_vec();
        }
        let mut buffer = vec![0i8; size];
        unsafe { std::ptr::copy_nonoverlapping(ptr, buffer[0..].as_mut_ptr(), size) }
        buffer
    }
}

impl Value<String> for String {
    fn data(&self, ptr: *const i8, _size: usize) -> String {
        if ptr.is_null() {
            return self.to_string();
        }
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    }
}

#[macro_export]
macro_rules! QR {
    ( $qr:expr, $( $f:expr ),* ) => {
        {
        let num_rows = $qr.num_rows() as usize;
        let num_fields = $qr.num_fields() as usize;
        let mut vec_result = Vec::with_capacity(num_rows);
        loop {
            let row = $qr.fetch_row();
            if row.is_null() {
                break;
            }
            let fetch_lengths = $qr.fetch_lengths() as *const u64;
            let val_size_array = ptr::slice_from_raw_parts(fetch_lengths, num_fields as usize);
            let row_val_array = ptr::slice_from_raw_parts(row, num_fields as usize);
            let mut idx = 0;
            vec_result.push(
                (
                $(
                    {
                    let val = unsafe { &*row_val_array }[idx];
                    let val_size = unsafe { &*val_size_array }[idx];
                    idx += 1;
                    let vd = Value::data(&$f, val, val_size as usize);
                    vd
                    },
                )*
                )
            );
            /*
            for idx in 0..num_fields as usize {
                let val = unsafe { &*row_val_array }[idx];
                if val.is_null() {
                    continue;
                }
                let val_size = unsafe { &*val_size_array }[idx];
                // mysql 是文本协议 整数也是字符串
            }
            */
        }
        vec_result
        }
    };
}
*/

fn main() {
    //diesel
    println!("hello mysql world");
    //println!("x:{} b:{}", v[0].0, v[0].1);

    test();

    let mut config = Config::new();

    config.set_host(&"127.0.0.1".into());
    config.set_user(&"root".into());
    config.set_password(&"root".into());
    config.set_database(&"dev_db".into());

    let mysql_connect = Connect::new(&config).unwrap();
    if let Err(err) = mysql_connect.connect() {
        println!("connect err:{}", err);
    }

    if let Err(err) = mysql_connect.set_mysql_options() {
        println!("set_mysql_options err:{}", err);
    }
    /*
    let sql = format!(
        "insert into test (id,name,text)values({},'name{}','text{}');",
        1, 1, 1
    );
    println!("sql:{}", sql);
    if let Err(err) = mysql_connect.alter_data(&sql) {
        println!("mysql_connect.alter_data err:{}", err);
    }
    */

    match mysql_connect.query_data(&"SELECT * FROM dev_db.test LIMIT 10;".into()) {
        Ok(result1) => {
            match mysql_connect.query_data(&"SELECT * FROM dev_db.test LIMIT 5;".into()) {
                Ok(result2) => {
                    let v1: Vec<i8> = vec![];
                    let v = mysql::result::QR!(&result2, 1i32, String::from(""), v1);

                    println!("{:?}", v);
                }
                Err(err) => {
                    println!("mysql_connect.query_data err:{}", err);
                }
            }
            //query_result(&result1);
        }
        Err(err) => {
            println!("mysql_connect.query_data err:{}", err);
        }
    }
}

#[inline]
fn c_str_to_string(ptr: *const i8) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    //.into_owned()
}

#[inline]
fn c_str_to_vec(ptr: *const i8, size: usize) -> Vec<i8> {
    let mut buffer = vec![0i8; size];
    unsafe { std::ptr::copy_nonoverlapping(ptr, buffer[0..].as_mut_ptr(), size) }
    buffer
}

fn query_result(query_result: &QueryResult) {
    let num_fields = query_result.num_fields() as usize;
    loop {
        let row = query_result.fetch_row();
        if row.is_null() {
            break;
        }
        let fetch_lengths = query_result.fetch_lengths() as *const u64;
        let val_size_array = ptr::slice_from_raw_parts(fetch_lengths, num_fields as usize);

        let row_val_array = ptr::slice_from_raw_parts(row, num_fields as usize);

        for idx in 0..num_fields as usize {
            let val = unsafe { &*row_val_array }[idx];
            if val.is_null() {
                continue;
            }

            let val_size = unsafe { &*val_size_array }[idx];
            // mysql 是文本协议 整数也是字符串
            if idx == 0 {
                let cs = c_str_to_string(val);
                match cs.parse::<i32>() {
                    Ok(num) => {
                        println!("val:{}  size:{} ---", num, val_size);
                    }
                    Err(err) => {
                        println!("err:{}  size:{} ---", err.to_string(), val_size);
                    }
                }
            } else {
                //在字符串中 '\0' 后面的字符会丢失 得想法解决
                println!("val:{}  size:{} ---", c_str_to_string(val), val_size);

                //let buffer = c_str_to_vec(val, val_size as usize);
                //println!("buffer:{:?}  size:{} ---", buffer, val_size);
            }
        }
        println!("");
    }
}

/*
fn query_result(query_result: &QueryResult) {
    let num_fields = query_result.num_fields() as usize;
    loop {
        let row = query_result.fetch_row();
        if row.is_null() {
            break;
        }
        let fetch_lengths = query_result.fetch_lengths() as *const u64;
        let val_size_array = ptr::slice_from_raw_parts(fetch_lengths, num_fields as usize);

        let row_val_array = ptr::slice_from_raw_parts(row, num_fields as usize);

        for idx in 0..num_fields as usize {
            let val = unsafe { &*row_val_array }[idx];
            if val.is_null() {
                continue;
            }

            let val_size = unsafe { &*val_size_array }[idx];
            // mysql 是文本协议 整数也是字符串
            if idx == 0 {
                let cs = c_str_to_string(val);
                match cs.parse::<i32>() {
                    Ok(num) => {
                        println!("val:{}  size:{} ---", num, val_size);
                    }
                    Err(err) => {
                        println!("err:{}  size:{} ---", err.to_string(), val_size);
                    }
                }
            } else {
                //在字符串中 '\0' 后面的字符会丢失 得想法解决
                println!("val:{}  size:{} ---", c_str_to_string(val), val_size);

                //let buffer = c_str_to_vec(val, val_size as usize);
                //println!("buffer:{:?}  size:{} ---", buffer, val_size);
            }
        }
        println!("");
    }
    */
