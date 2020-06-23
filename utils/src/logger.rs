use crate::time;
use log;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Logger /*<W: Write + Send + 'static>*/ {
    file_path: String,
    level: log::Level,
    file: Mutex<File>,
    new_file_interval: u64,
    open_file_ts: Arc<AtomicU64>, //Arc<RefCell<u64>> 多线程不能用
}

#[inline]
fn new_file_path(file_path: &String, time: &time::Time) -> String {
    format!(
        "{}_{}-{:0>2}-{:0>2}_{:0>2}-{:0>2}-{:0>2}",
        file_path,
        time.year,
        time.month + 1,
        time.day,
        time.hour + 8,
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
        time.hour + 8,
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
    pub fn init(level: &String, file_path: &String, new_file_interval: u16) -> Result<(), String> {
        let log_level = match level.to_uppercase().as_str() {
            "TRACE" => log::Level::Trace,
            "DEBUG" => log::Level::Debug,
            "INFO" => log::Level::Info,
            "WARN" => log::Level::Warn,
            "ERROR" => log::Level::Error,
            _ => log::Level::Error,
        };

        let ts = time::timestamp();
        let path = Path::new(&file_path);
        if let Some(dir) = path.parent() {
            match fs::create_dir_all(&dir) {
                Ok(()) => (),
                Err(err) => return Err(format!("create_dir_all:{} error:{}", file_path, err)),
            }
        }

        let mut new_file_duration = 1;
        if new_file_interval > 0 {
            new_file_duration = new_file_interval;
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
                    open_file_ts: Arc::new(AtomicU64::new(ts)),
                    new_file_interval: new_file_duration as u64 * 60 * 60 * 1000
                });

                match log::set_boxed_logger(logger) {
                    Err(err) => Err(format!("{}", err)),
                    Ok(()) => Ok(log::set_max_level(log_level.to_level_filter())),
                }
            }
            Err(err) => Err(format!("file:{} error:{}", file_path, err)),
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
            let ts = time::timestamp();
            let time = time::timestamp_to_time(ts);
            if let Err(_err) = file_lock.write(fmt_log(&record, &time).as_bytes()) {
                return;
            }

            //可优化 定时flush
            if let Err(_err) = file_lock.flush() {
                return;
            }

            if self.open_file_ts.load(Ordering::Relaxed) + self.new_file_interval < ts {
                if let Err(_err) = file_lock.flush() {
                    return;
                }
                let open_file_ts = self.open_file_ts.load(Ordering::Relaxed);
                let open_file_time = time::timestamp_to_time(open_file_ts);
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
                        self.open_file_ts.store(ts, Ordering::Relaxed);
                    }
                    Err(err) => println!("create log file ({}) error:{}", &self.file_path, err),
                };
            }
        }
    }

    //定时 flush
    fn flush(&self) {
        if let Ok(mut file_lock) = self.file.lock() {
            if let Err(_err) = file_lock.flush() {
                return;
            }
        }
    }
}
