use crate::time;
use log;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

const HOUR_8: u64 = 8 * 60 * 60 * 1000;

pub struct Logger /*<W: Write + Send + 'static>*/ {
    file_path: String,
    level: log::Level,
    file: Mutex<File>,
}

#[inline]
fn new_file_path(file_path: &String, time: &time::Time) -> String {
    format!(
        "{}_{}-{:0>2}-{:0>2}_{:0>2}-{:0>2}-{:0>2}",
        file_path,
        time.year,
        time.month + 1,
        time.day,
        time.hour,
        time.min,
        time.sec
    )
}

#[inline]
fn fmt_log(record: &log::Record, time: &time::Time) -> String {
    let line = match record.line() {
        Some(line) => line,
        None => 0,
    };

    let file = match record.file() {
        Some(file) => file,
        None => "unkonw file",
    };

    format!(
        "{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3} {}:{}->{}|{}\n",
        time.year,
        time.month + 1,
        time.day,
        time.hour,
        time.min,
        time.sec,
        time.ms,
        record.level(),
        file,
        line,
        record.args()
    )
}

impl Logger /*<W>*/ {
    //new_file_interval:单位小时
    pub fn init(level: &String, file_path: &String) -> Result<(), String> {
        let log_level = match level.to_uppercase().as_str() {
            "TRACE" => log::Level::Trace,
            "DEBUG" => log::Level::Debug,
            "INFO" => log::Level::Info,
            "WARN" => log::Level::Warn,
            "ERROR" => log::Level::Error,
            _ => log::Level::Error,
        };

        let path = Path::new(&file_path);
        if let Some(dir) = path.parent() {
            match fs::create_dir_all(&dir) {
                Ok(()) => (),
                Err(err) => return Err(format!("create_dir:{:?} error:{}", dir, err)),
            }
        }
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_path)
        {
            Ok(file) => {
                let logger = Box::new(Logger {
                    level: log_level,
                    file: Mutex::new(file),
                    file_path: file_path.clone(),
                });

                match log::set_boxed_logger(logger) {
                    Err(err) => Err(format!("{}", err)),
                    Ok(()) => Ok(log::set_max_level(log_level.to_level_filter())),
                }
            }
            Err(err) => Err(format!("open:{} error:{}", &file_path, err)),
        }
    }
}

impl log::Log for Logger /*<W>*/ {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if let Ok(mut file_lock) = self.file.lock() {
            let time = time::timestamp_to_time(time::timestamp() + HOUR_8);
            if let Err(_err) = file_lock.write(fmt_log(&record, &time).as_bytes()) {
                return;
            }

            /*
            //可优化 定时flush
            if let Err(_err) = file_lock.flush() {
                return;
            }
            */
        }
    }

    //定时 flush
    fn flush(&self) {
        if let Ok(mut file_lock) = self.file.lock() {
            if let Err(_err) = file_lock.flush() {
                return;
            }

            let now_timestame = time::timestamp() + HOUR_8;
            let open_file_time = time::timestamp_to_time(now_timestame);
            let new_file_path = new_file_path(&self.file_path, &open_file_time);

            if let Err(_) = fs::rename(&self.file_path, &new_file_path) {
                return;
            }

            match OpenOptions::new()
                .append(true)
                .create(true)
                .open(&self.file_path)
            {
                Ok(file) => {
                    *file_lock = file;
                }
                Err(err) => println!("open file:{} error:{}", &self.file_path, err),
            };
        }
    }
}
