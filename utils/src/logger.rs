use log::*;

struct Logger {
    level: Level,
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            return
        }

        match self.writable.lock(){
            Ok(mut write_lock)=>
                match writeln!(
                    write_lock,
                    "{}:{}--{}",
                    record.level(),
                    record.target(),
                    record.args()
                ){
                    Ok(_)=> 0,
                    Err(_)=> 1
                }
            Err(_)=> err
        }
        

        println!(
            "{}:{} -- {}",
            record.level(),
            record.target(),
            record.args()
        );
    }

    fn flush(&self) {
        match self.writable.lock(){
            Ok(mut write_lock)=>
            match write_lock.flush() {
                Ok(_)=> 0,
                Err(_)=> 1
            }
        }
    }
}
