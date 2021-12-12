extern crate ini;
use ini::Ini;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::{thread, time};

// Global variables
const NEW_CONFIG_FILE_MODE: u8 = 0;
pub const _CURRENT_CONFIG_FILE_MODE: u8 = 1;

// Default values
const CONFIG_FILE: &str = "config/config.ini";
const DEFAULT_LOG_DIRECTORY: &str = "logs";
const DEFAULT_PORT: u16 = 6969;
const DEFAULT_DATABASE: &str = "ruslydb";
const DEFAULT_USER: &str = "rusly";
const DEFAULT_PASSWD: &str = "root";

#[derive(PartialEq)]
pub struct Config {
    pub port: u16,
    pub logs_directory: String,
    pub database: String,
    pub user: String,
    pub passwd: String,
}

// Implement the clone function to the struct Config
impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            port: self.port.clone(),
            logs_directory: self.logs_directory.clone(),
            database: self.database.clone(),
            user: self.user.clone(),
            passwd: self.passwd.clone(),
        }
    }
}

// This function checks if the config file exists
pub fn check_init_file() -> Config {
    if Path::new(CONFIG_FILE).exists() == true {
        return parse_init_file();
    } else {
        let config = Config {
            port: DEFAULT_PORT,
            logs_directory: DEFAULT_LOG_DIRECTORY.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            user: DEFAULT_USER.to_string(),
            passwd: DEFAULT_PASSWD.to_string(),
        };
        create_new_config_file(NEW_CONFIG_FILE_MODE, config.clone());
        return config;
    }
}

// This function is charged to parse the init file and extract config values
fn parse_init_file() -> Config {
    let mut error: bool = false;

    let conf = Ini::load_from_file(CONFIG_FILE).unwrap();
    let network_section = conf.section(Some("NETWORK SETTINGS")).unwrap();
    let port = network_section.get("port").unwrap();

    let logs_section = conf.section(Some("LOGS SETTINGS")).unwrap();
    let logs_directory = logs_section.get("directory").unwrap();

    let database_section = conf.section(Some("DATABASE SETTINGS")).unwrap();
    let database = database_section.get("database").unwrap();
    let user = database_section.get("user").unwrap();
    let passwd = database_section.get("passwd").unwrap();

    if check_port(port.to_string()) == false && error != true {
        error = true;
        error_at_startup("Error: The destination port in the config file is not valid");
    }
    if error == false {
        return Config {
            port: port.parse::<u16>().unwrap(),
            logs_directory: logs_directory.to_string(),
            database: database.to_string(),
            user: user.to_string(),
            passwd: passwd.to_string(),
        };
    } else {
        return Config {
            port: DEFAULT_PORT,
            logs_directory: DEFAULT_LOG_DIRECTORY.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            user: DEFAULT_USER.to_string(),
            passwd: DEFAULT_PASSWD.to_string(),
        };
    }
}

// This function is charged to warn the user that the config file is invalid/corrupted
fn error_at_startup(error_msg: &str) {
    println!("{}", error_msg);
    println!("Press n to generate new config file, otherwise RuslyChat will shutdown :/");
    let mut buff = String::new();
    io::stdin()
        .read_line(&mut buff)
        .expect("Reading from stdin failed");
    buff = buff.trim().to_string();

    if buff == String::from("n") {
        let config = Config {
            port: DEFAULT_PORT,
            logs_directory: DEFAULT_LOG_DIRECTORY.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            user: DEFAULT_USER.to_string(),
            passwd: DEFAULT_PASSWD.to_string(),
        };
        create_new_config_file(NEW_CONFIG_FILE_MODE, config);
    } else {
        std::process::exit(1);
    }
}

// This functions is charged to create / update the config file
fn create_new_config_file(mode: u8, config: Config) {
    if mode == NEW_CONFIG_FILE_MODE {
        if Path::new("config").exists() == false {
            let f = fs::create_dir_all("config");
            match f {
                Err(e) => {
                    println!("Error, the config file can't be created\n{}", e);
                    let duration = time::Duration::from_secs(2);
                    thread::sleep(duration);
                    process::exit(0);
                }
                _ => (),
            }
        }
    }

    let mut conf = Ini::new();
    conf.with_section(Some("NETWORK SETTINGS"))
        .set("port", config.port.to_string());
    conf.with_section(Some("LOGS SETTINGS"))
        .set("directory", config.logs_directory);
    conf.with_section(Some("DATABASE SETTINGS"))
        .set("database", config.database)
        .set("user", config.user)
        .set("passwd", config.passwd);
    conf.write_to_file(CONFIG_FILE).unwrap();
}

// This function is charged to verify the destination port value
fn check_port(buff: String) -> bool {
    let new_port: u16;
    match buff.parse::<u16>() {
        Err(_) => {
            return false;
        }
        _ => {
            new_port = buff.parse::<u16>().unwrap();
            if new_port <= 1024 {
                return false;
            } else {
                return true;
            }
        }
    }
}
