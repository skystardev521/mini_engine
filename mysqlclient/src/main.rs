use mysqlclient::Config;
use mysqlclient::Connect;
use mysqlclient::QueryResult;
use mysqlclient::Value;
use std::any::TypeId;
use std::ffi::CStr;
use std::ptr::{self};
use utils::logger::Logger;
use utils::time;

use mysqlclient::DbTask;

#[macro_use]
mod result;

struct test{

}


fn main() {

    let t = test{};
    let t1 = t.as_ptr();

    let mut task: DbTask<u64> = DbTask::new(
        "db_name".into(),
        "sql_str".into(),
        Box::new(|result| match result {
            Ok(val) => {
                println!("xxxxxxxxxxxx:{}", val);
            }
            Err(err) => {
                println!("xxxxxxxxxxxx:{}", err);
            }
        }),
    );
    task.result = Ok(1234567);
    (task.callback)(&task.result);

    println!("------------------------------------------------");
    //println!("{:?}", tuple_default!(i32, String));

    println!("------------------------------------------------");

    let mut open_log_file_ts = time::timestamp();
    match Logger::init(&String::from("info"), &String::from("logs/mysql.log")) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

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

    let sql = format!(
        "insert into test (id,name,text)values({},'name{}','text{}');",
        1, 1, 1
    );
    println!("sql:{}", sql);
    match mysql_connect.alter_data(&sql) {
        Ok(row_num) => {
            println!("mysql_connect.alter_data row num:{}", row_num);
        }
        Err(err) => {
            println!("mysql_connect.alter_data err:{}", err);
        }
    }

    match mysql_connect.query_data(&"SELECT * FROM dev_db.test LIMIT 10;".into()) {
        Ok(result1) => {
            match mysql_connect
                .query_data(&"SELECT * FROM dev_db.test where id = 11111 LIMIT 5;".into())
            {
                Ok(result2) => {
                    let result = query_result!(&result2, 0i32, String::new(), vec![0i8; 0]);
                    if result.is_empty() {
                        println!("result is_empty");
                    } else {
                        println!("result:{:?}", result);
                    }
                }
                Err(err) => {
                    println!("mysql_connect.query_data err:{}", err);
                }
            }
            let result = query_result!(&result1, 0i32, String::new(), vec![0i8; 0]);
            println!("----------------------------------------------------------");
            if result.is_empty() {
                println!("result is_empty");
            } else {
                println!("result:{:?}", result);
            }
        }
        Err(err) => {
            println!("mysql_connect.query_data err:{}", err);
        }
    }
}
