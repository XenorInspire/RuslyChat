extern crate ini;
use ini::Ini;
use regex::Regex;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::{thread, time};

// Global variables
const NEW_CONFIG_FILE_MODE: u8 = 0;
pub const CURRENT_CONFIG_FILE_MODE: u8 = 1;

// Default values
const CONFIG_FILE: &str = "config/config.ini";
const DEFAULT_LOG_DIRECTORY: &str = "logs";
const DEFAULT_DESTINATION_PORT: u16 = 6969;
const DEFAULT_DOMAIN: &str = "127.0.0.1";

// Marvelous regex <3
const IPV4_REGEX_SYNTAX: &str =
    r"(\b25[0-5]|\b2[0-4][0-9]|\b[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}";
const IPV6_REGEX_SYNTAX: &str = r"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))";
const DOMAIN_REGEX_SYNTAX: &str = r"[a-zA-Z0-9][a-zA-Z0-9-]{1,61}[a-zA-Z0-9](?:\.[a-zA-Z]{2,})+";

#[derive(PartialEq)]
pub struct Config {
    pub domain: String,
    pub port_dest: u16,
    pub logs_directory: String,
}

// Implement the clone function to the struct Config
impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            domain: self.domain.clone(),
            port_dest: self.port_dest.clone(),
            logs_directory: self.logs_directory.clone(),
        }
    }
}

// This function checks if the config file exists
pub fn check_init_file() -> Config {
    if Path::new(CONFIG_FILE).exists() == true {
        return parse_init_file();
    } else {
        let config = Config {
            domain: DEFAULT_DOMAIN.to_string(),
            port_dest: DEFAULT_DESTINATION_PORT,
            logs_directory: DEFAULT_LOG_DIRECTORY.to_string(),
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
    let domain = network_section.get("domain").unwrap();
    let port_dest = network_section.get("portDest").unwrap();

    let logs_section = conf.section(Some("LOGS SETTINGS")).unwrap();
    let logs_directory = logs_section.get("directory").unwrap();

    if check_domain(domain.to_string()) == false {
        error = true;
        error_at_startup("Error: The IP adrress in the config file is not valid");
    }

    if check_destinaton_port(port_dest.to_string()) == false && error != true {
        error = true;
        error_at_startup("Error: The destination port in the config file is not valid");
    }

    if Path::new(logs_directory).exists() == false && error != true {
        error = true;
        error_at_startup("Error: The log directory in the config file is not valid");
    }

    if error == false {
        return Config {
            domain: domain.to_string(),
            port_dest: port_dest.parse::<u16>().unwrap(),
            logs_directory: logs_directory.to_string(),
        };
    } else {
        return Config {
            domain: DEFAULT_DOMAIN.to_string(),
            port_dest: DEFAULT_DESTINATION_PORT,
            logs_directory: DEFAULT_LOG_DIRECTORY.to_string(),
        };
    }
}

fn error_at_startup(error_msg: &str) {
    println!("{}", error_msg);
    println!("Press n to generate new config file, otherwise, RuslyChat will shutdown :/");
    let mut buff = String::new();
    io::stdin()
        .read_line(&mut buff)
        .expect("Reading from stdin failed");
    buff = buff.trim().to_string();

    if buff == String::from("n") {
        let config = Config {
            domain: DEFAULT_DOMAIN.to_string(),
            port_dest: DEFAULT_DESTINATION_PORT,
            logs_directory: DEFAULT_LOG_DIRECTORY.to_string(),
        };
        create_new_config_file(NEW_CONFIG_FILE_MODE, config);
    } else {
        std::process::exit(1);
    }
}

// This functions is charged to create / update the config file
pub fn create_new_config_file(mode: u8, config: Config) {
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

        if Path::new("logs").exists() == false {
            let f = fs::create_dir_all("logs");
            match f {
                Err(e) => {
                    println!("Error, the log directory can't be created\n{}", e);
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
        .set("domain", config.domain)
        .set("portDest", config.port_dest.to_string());
    conf.with_section(Some("LOGS SETTINGS"))
        .set("directory", config.logs_directory);
    conf.write_to_file(CONFIG_FILE).unwrap();
}

// The settings menu (to edit some parameters)
pub fn change_config_values(mut temp_config: Config) -> Config {
    let mut answer = String::from("1");

    while answer.eq("0") == false {
        std::process::Command::new("clear").status().unwrap();

        display_settings_menu(&temp_config);

        println!("Select setting to change value");
        println!("1 : Domain (or IP address)");
        println!("2 : Destination port");
        println!("3 : Logs directory");
        println!("0 : Exit settings");

        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        match &*answer {
            "1" => {
                let temp = change_domain();
                if temp != String::from("q") {
                    temp_config.domain = temp;
                }
            }
            "2" => {
                let temp = change_destination_port();
                if temp != 0 {
                    temp_config.port_dest = temp;
                }
            }
            "3" => {
                let temp = change_logs_directory();
                if temp != String::from("q") {
                    temp_config.logs_directory = temp;
                }
            }
            _ => (),
        }
    }

    return temp_config;
}

// Kind of obvious...
fn display_settings_menu(config: &Config) {
    std::process::Command::new("clear").status().unwrap();

    println!("========================\n        Settings         \n========================");
    println!("Domain => {}", config.domain);
    println!("Destination port => {}", config.port_dest);
    println!("Log directory => {}\n", config.logs_directory);
}

// This function is charged to check the domain value entered by the user
fn change_domain() -> String {
    let mut buff = String::new();

    while check_domain(buff.clone()) == false && buff != String::from("q") {
        std::process::Command::new("clear").status().unwrap();

        buff = String::from("");
        println!("Enter a valid domain (or IP address)");
        println!("Press q to cancel");
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        buff = buff.trim().to_string();
    }

    return buff;
}

// This function is charged to check the destination port value entered by the user
fn change_destination_port() -> u16 {
    let mut buff = String::from("");

    while check_destinaton_port(buff.clone()) == false {
        std::process::Command::new("clear").status().unwrap();

        println!("Enter a valid destination port");
        println!("Press q to cancel");
        buff = String::from("");

        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        buff = buff.trim().to_string();

        if buff == String::from("q") {
            return 0;
        }
    }

    return buff.parse::<u16>().unwrap();
}

// This function is charged to check the log directory value entered by the user
fn change_logs_directory() -> String {
    let mut buff = String::new();

    while Path::new(&*buff).exists() == false && buff != String::from("q") {
        println!("Enter a valid path for the log directory");
        println!("Press q to cancel");

        buff = String::from("");
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        buff = buff.trim().to_string();
    }

    return buff;
}

// This function is charged to verify the domain value
fn check_domain(buff: String) -> bool {
    let ipv4_regex = Regex::new(IPV4_REGEX_SYNTAX).unwrap();
    let ipv6_regex = Regex::new(IPV6_REGEX_SYNTAX).unwrap();
    let domain_regex = Regex::new(DOMAIN_REGEX_SYNTAX).unwrap();
    if ipv4_regex.is_match(&*buff) == false
        && ipv6_regex.is_match(&*buff) == false
        && domain_regex.is_match(&*buff) == false
        && buff != String::from("localhost")
    {
        return false;
    } else {
        return true;
    }
}

// This function is charged to verify the destination port value
fn check_destinaton_port(buff: String) -> bool {
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
