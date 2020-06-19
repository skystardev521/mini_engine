use log;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

pub struct Logger /*<W: Write + Send + 'static>*/ {
    level: log::Level,
    file: std::sync::Mutex<File>,
}

impl Logger /*<W>*/ {
    pub fn init(level: &String, path: &String) -> Result<(), String> {
        let log_level = match level.to_uppercase().as_str() {
            "TRACE" => log::Level::Trace,
            "DEBUG" => log::Level::Debug,
            "INFO" => log::Level::Info,
            "WARN" => log::Level::Warn,
            "ERROR" => log::Level::Error,
            _ => log::Level::Error,
        };

        match OpenOptions::new().append(true).open(path) {
            Ok(file) => {
                let logger = Box::new(Logger {
                    level: log_level,
                    file: std::sync::Mutex::new(file),
                });

                match log::set_boxed_logger(logger) {
                    Ok(()) => Ok(log::set_max_level(log_level.to_level_filter())),
                    Err(err) => Err(format!("{}", err)),
                }
            }
            Err(err) => Err(format!("{}", err)),
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

                let wresult = file_lock.write_fmt(format_args!(
                    "{}:{}->{}|{}\n",
                    record.level(),
                    file,
                    line,
                    record.args()
                ));

                match wresult {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
            Err(_) => -1,
        };
        let line = match record.line() {
            Some(n) => n,
            None => 0,
        };

        let file = match record.file() {
            Some(str) => str,
            None => "unkonw file",
        };

        print!("{}:{}->{}|{}\n", record.level(), file, line, record.args());

        self.flush();
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
