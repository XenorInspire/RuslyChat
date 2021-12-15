use chrono::{DateTime, Utc};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::{env, fs};

// This struct list all type of logs
pub enum LogLevel {
    FATAL,
    ERROR,
    INFO,
    TRACE,
    DEBUG,
}

pub struct Logger {
    pub path: String,
    pub log_file: String,
}

// Get a logger where you want
pub fn get_logger() -> Logger {
    let path_logger;
    let log_file;
    match env::var("PATH_LOGGER") {
        Ok(val) => path_logger = val,
        Err(_e) => path_logger = "logs".to_string(),
    }

    match env::var("LOG_FILE") {
        Ok(val) => log_file = val,
        Err(_e) => {
            log_file = get_log_file_name(path_logger.clone());
            env::set_var("LOG_FILE", log_file.clone());
        }
    }

    Logger {
        path: path_logger.clone(),
        log_file: log_file.clone(),
    }
}

impl Logger {
    pub fn log(&mut self, message: String, flag: LogLevel) {
        if check_log_directory(self.path.clone()) {
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

            to_log += &*(get_log_time() + " : " + &*message + "\n");

            print!("{}", to_log);

            match file.write_all(to_log.as_bytes()) {
                Err(e) => {
                    println!("Error, can not write into {}\n{}", self.log_file, e);
                }
                _ => (),
            };
        }
    }
}

// Get current timestamp
fn get_log_time() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("[%d-%m-%Y %H:%M:%S]").to_string()
}

// Build the name of the log file
fn get_log_file_name(path: String) -> String {
    let now: DateTime<Utc> = Utc::now();
    let time = now.format("%Y%m%d%H%M%S").to_string();
    path + "/ruslychat_" + &*time + ".log"
}

// Kind of obvious...
pub fn check_log_directory(path: String) -> bool {
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
