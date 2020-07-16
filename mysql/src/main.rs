use mysql::Config;
use mysql::MysqlConnect;
use std::ptr::{self, NonNull};

use mysql::test;

fn main() {
    //diesel
    println!("hello mysql world");

    test();
    

    let mut config = Config::new();

    config.set_host(&"127.0.0.1".into());
    config.set_user(&"root".into());
    config.set_password(&"root".into());
    config.set_database(&"dev_db".into());

    let mysql_connect = MysqlConnect::new(&config).unwrap();
    if let Err(err) = mysql_connect.connect() {
        println!("connect err:{}", err);
    }

    if let Err(err) = mysql_connect.set_mysql_options() {
        println!("set_mysql_options err:{}", err);
    }

    if let Err(err) = mysql_connect.real_query(&"SELECT * FROM dev_db.test LIMIT 20;".into()) {
        println!("mysql_connect.execute err:{}", err);
    }

    let _fields = mysql_connect.fetch_fields();

    let field_num = mysql_connect.num_fields();
    let _num_rows = mysql_connect.num_rows();
    loop {
        let row = mysql_connect.fetch_row();
        if row.is_null() {
            break;
        }

        let slice = ptr::slice_from_raw_parts(row, field_num as usize);

        for f in 0..field_num as usize {
            let p = unsafe { &*slice }[f];
            if f == 0 {
                print!("{:?} ", unsafe { *(p as *const u16) });
            } else {
                let val = c_str_to_string(p);
                print!("{} ", val);
            }
        }
        println!("");
        //}
    }

    fn c_str_to_string(ptr: *const i8) -> String {
        unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

