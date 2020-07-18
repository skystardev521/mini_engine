use mysql::test;
use mysql::Config;
use mysql::Connect;
use mysql::QueryResult;
use std::ffi::CStr;
use std::ptr::{self, NonNull};

fn main() {
    //diesel
    println!("hello mysql world");

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
            match mysql_connect.query_data(&"SELECT * FROM dev_db.test LIMIT 2;".into()) {
                Ok(result2) => {
                    query_result(&result2);
                }
                Err(err) => {
                    println!("mysql_connect.query_data err:{}", err);
                }
            }
            query_result(&result1);
        }
        Err(err) => {
            println!("mysql_connect.query_data err:{}", err);
        }
    }
}

fn c_str_to_string(ptr: *const i8) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    //.into_owned()
}

fn query_result(query_result: &QueryResult) {
    let _fields = query_result.fetch_fields();
    let field_num = query_result.num_fields();
    let _num_rows = query_result.num_rows();
    loop {
        let row = query_result.fetch_row();
        if row.is_null() {
            break;
        }
        let lenghts = query_result.fetch_lengths() as * const u64;
        let val_size_array = ptr::slice_from_raw_parts(lenghts, field_num as usize);

        let row_val_array = ptr::slice_from_raw_parts(row, field_num as usize);

        for f_idx in 0..field_num as usize {
            let val = unsafe { &*row_val_array }[f_idx];
            let val_size = unsafe { &*val_size_array }[f_idx];
            // mysql 是文本协议 整数也是字符串
            unsafe {
                //在字符串中 '\0' 后面的字符会丢失 得想法解决
                print!("val:{}  size:{} ---", c_str_to_string(val), val_size);
            }
        }
        println!("");
    }
}
