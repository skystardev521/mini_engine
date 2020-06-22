use crate::time;
use log;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

pub struct Logger /*<W: Write + Send + 'static>*/ {
    now_logs: Arc<AtomicU32>,
    max_logs: u32,
    file_path: String,
    level: log::Level,
    file: Arc<Mutex<File>>,
}

pub fn new_file_path(file_path: &String) -> String {
    let now = time::now_time();
    format!(
        "{}_{}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}",
        file_path,
        now.year,
        now.month + 1,
        now.day,
        now.hour + 8,
        now.min,
        now.sec
    )
}

impl Logger /*<W>*/ {
    //max_logs:file max logs
    pub fn init(level: &String, file_path: String, max_logs: u32) -> Result<(), String> {
        let log_level = match level.to_uppercase().as_str() {
            "TRACE" => log::Level::Trace,
            "DEBUG" => log::Level::Debug,
            "INFO" => log::Level::Info,
            "WARN" => log::Level::Warn,
            "ERROR" => log::Level::Error,
            _ => log::Level::Error,
        };
        let new_file_path = new_file_path(&file_path);
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(new_file_path)
        {
            Ok(file) => {
                let logger = Box::new(Logger {
                    now_logs: Arc::new(AtomicU32::new(0)),
                    max_logs: max_logs,
                    file_path: file_path,
                    level: log_level,
                    file: Arc::new(Mutex::new(file)),
                });

                match log::set_boxed_logger(logger) {
                    Ok(()) => Ok(log::set_max_level(log_level.to_level_filter())),
                    Err(err) => Err(format!("{}", err)),
                }
            }
            Err(err) => Err(format!("{}", err)),
        }
    }
    /*
    pub fn new_log_file(&self, lock_file: &mut MutexGuard<File>) {
        self.now_logs.store(0, Ordering::Relaxed);
        let file_path = Logger::new_file_path(&self.file_path);
        match OpenOptions::new().append(true).open(file_path) {
            Ok(file) => *lock_file = file,
            Err(_) => (),
        };
    }

    pub fn check_logs(&self, lock_file: MutexGuard<File>) {
        self.now_logs.fetch_add(1, Ordering::Relaxed);
        if self.now_logs.load(Ordering::Relaxed) > self.max_logs {
            println!("xxxxxxxxxxxxxxxxxxxxxxxxxxx:{}", self.max_logs);
            self.new_log_file(lock_file);
        }
    }
    */
}

impl log::Log for Logger /*<W>*/ {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match self.file.lock() {
            Ok(mut file_lock) => {
                let line = match record.line() {
                    Some(n) => n,
                    None => 0,
                };

                let file = match record.file() {
                    Some(str) => str,
                    None => "unkonw file",
                };

                let now = time::now_time();
                let wresult = file_lock.write_fmt(format_args!(
                    "{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3} {}:{}->{}|{}\n",
                    now.year,
                    now.month + 1,
                    now.day,
                    now.hour + 8,
                    now.min,
                    now.sec,
                    now.ms,
                    record.level(),
                    file,
                    line,
                    record.args()
                ));

                match wresult {
                    Ok(_) => match file_lock.flush() {
                        Ok(()) => {
                            self.now_logs.fetch_add(1, Ordering::Relaxed);
                            if self.now_logs.load(Ordering::Relaxed) > self.max_logs {
                                //new log file
                                self.now_logs.store(0, Ordering::Relaxed);
                                let file_path = new_file_path(&self.file_path);

                                println!("new log file :{}", &file_path);
                                match OpenOptions::new()
                                    .append(true)
                                    .create(true)
                                    .open(&file_path)
                                {
                                    Ok(file) => *file_lock = file,
                                    Err(err) => {
                                        println!("create log file ({}) error:{}", &file_path, err)
                                    }
                                };
                            }
                        }
                        Err(_) => (),
                    },
                    Err(_) => (),
                }
            }
            Err(_) => (),
        };

        //------------- print---------------------

        let line = match record.line() {
            Some(n) => n,
            None => 0,
        };

        let file = match record.file() {
            Some(str) => str,
            None => "unkonw file",
        };
        let now = time::now_time();
        print!(
            "{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3} {}:{}->{}|{}\n",
            now.year,
            now.month + 1,
            now.day,
            now.hour + 8,
            now.min,
            now.sec,
            now.ms,
            record.level(),
            file,
            line,
            record.args()
        );
    }

    fn flush(&self) {
        println!("log flush");
        match self.file.lock() {
            Ok(mut file_lock) => {
                match file_lock.flush() {
                    Ok(()) => (),
                    Err(_) => (),
                };
            }
            Err(_) => (),
        };
    }
}
