use std::path::Path;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{DateTime, Utc};

pub enum LogLevel {
    FATAL,
    ERROR,
    INFO,
    TRACE,
    DEBUG
}

pub struct Logger {
    pub path: String,
    pub log_file: String,
    pub max_size: u16
}

impl Logger {
    pub fn log(&mut self, message: String, flag: LogLevel) {
        if check_log_directory(self.path.clone()) {
            if self.log_file == "" {
                self.log_file = get_log_file_name(self.path.clone());
            }

            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(self.log_file.clone())
                .unwrap();

            let mut to_log;

            match flag {
                LogLevel::FATAL => to_log = "[FATAL]".to_string(),
                LogLevel::ERROR => to_log = "[ERROR]".to_string(),
                LogLevel::INFO => to_log = "[INFO]".to_string(),
                LogLevel::TRACE => to_log = "[TRACE]".to_string(),
                LogLevel::DEBUG => to_log = "[DEBUG]".to_string(),
            }

            to_log += &*(get_log_time() + " : " + &*message);

            match file.write_all(to_log.as_bytes()) {
                Err(e) => {
                    println!("Error, can not write into {}\n{}", self.log_file, e);
                }
                _ => (),
            };
        }
    }
}

fn get_log_time() -> String {
    let now: DateTime<Utc> = Utc::now();

    now.format("[%d-%m-%Y %H:%M:%S]").to_string()
}

fn get_log_file_name(path: String) -> String {
    let now: DateTime<Utc> = Utc::now();
    let time = now.format("%Y%m%d%H%M%S").to_string();

    path + "/ruslychat_"  + &*time + ".log"
}

fn check_log_directory(path: String) -> bool {
    if !Path::new(&path).exists() {
        match fs::create_dir_all(path) {
            Err(e) => {
                println!("Error, log directory can not be created\n{}", e);
                return false;
            }
            _ => (),
        };
    }

    true
}