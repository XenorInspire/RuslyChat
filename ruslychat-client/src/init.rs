extern crate ini;
use ini::Ini;
use regex::Regex;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::{thread, time};

const CONFIG_FILE: &str = "config/config.ini";
const NEW_CONFIG_FILE_MODE: u8 = 0;
pub const CURRENT_CONFIG_FILE_MODE: u8 = 1;

#[derive(PartialEq)]
pub struct Config {
    pub domain: String,
    pub port_dest: u16,
    pub logs_directory: String,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            domain: self.domain.clone(),
            port_dest: self.port_dest.clone(),
            logs_directory: self.logs_directory.clone(),
        }
    }
}

pub fn check_init_file() -> Config {
    if Path::new(CONFIG_FILE).exists() == true {
        return parse_init_file();
    } else {
        let config = Config {
            domain: "127.0.0.1".to_string(),
            port_dest: 6969,
            logs_directory: "logs".to_string(),
        };
        create_new_config_file(NEW_CONFIG_FILE_MODE, config.clone());
        return config;
    }
}

fn parse_init_file() -> Config {
    let conf = Ini::load_from_file(CONFIG_FILE).unwrap();
    let network_section = conf.section(Some("NETWORK SETTINGS")).unwrap();
    let domain = network_section.get("domain").unwrap();
    let port_dest = network_section.get("portDest").unwrap();

    let logs_section = conf.section(Some("LOGS SETTINGS")).unwrap();
    let logs_directory = logs_section.get("directory").unwrap();

    let config = Config {
        domain: domain.to_string(),
        port_dest: port_dest.parse::<u16>().unwrap(),
        logs_directory: logs_directory.to_string(),
    };

    return config;
}

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
    }

    let mut conf = Ini::new();
    conf.with_section(Some("NETWORK SETTINGS"))
        .set("domain", config.domain)
        .set("portDest", config.port_dest.to_string());
    conf.with_section(Some("LOGS SETTINGS"))
        .set("directory", config.logs_directory);
    conf.write_to_file(CONFIG_FILE).unwrap();
}

pub fn change_config_values(mut temp_config: Config) -> Config {
    display_settings_menu(&temp_config);
    let mut answer = String::from("1");

    while answer.eq("0") == false {
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
            "1" => temp_config.domain = change_domain(),
            "2" => temp_config.port_dest = change_destination_port(),
            "3" => temp_config.logs_directory = change_logs_directory(),
            _ => (),
        }
    }

    return temp_config;
}

fn display_settings_menu(config: &Config) {
    println!("========================\n        Settings         \n========================");
    println!("Domain => {}", config.domain);
    println!("Destination port => {}", config.port_dest);
    println!("Log directory => {}\n", config.logs_directory);
}

fn change_domain() -> String {
    let ipv4_regex = Regex::new(r"(\b25[0-5]|\b2[0-4][0-9]|\b[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}").unwrap();
    let ipv6_regex = Regex::new(r"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))").unwrap();
    let domain_regex =
        Regex::new(r"[a-zA-Z0-9][a-zA-Z0-9-]{1,61}[a-zA-Z0-9](?:\.[a-zA-Z]{2,})+").unwrap();
    let mut buff = String::new();

    while ipv4_regex.is_match(&*buff) == false
        && ipv6_regex.is_match(&*buff) == false
        && domain_regex.is_match(&*buff) == false
        && buff != String::from("localhost")
    {
        buff = String::from("");
        println!("Enter a valid domain (or IP address)");
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        buff = buff.trim().to_string();
    }

    return buff;
}

fn change_destination_port() -> u16 {
    let mut new_port: u16 = 0;

    while new_port <= 1024 {
        println!("Enter a valid destination port");
        let mut buff = String::from("");

        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        buff = buff.trim().to_string();

        match buff.parse::<u16>() {
            Err(e) => {
                new_port = 0;
                println!("Error: {}\n", e)
            }
            _ => new_port = buff.parse::<u16>().unwrap(),
        }
    }

    return new_port;
}

fn change_logs_directory() -> String {
    let mut buff = String::new();

    while Path::new(&*buff).exists() == false {
        println!("Enter a valid path for the log directory");

        buff = String::from("");
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        buff = buff.trim().to_string();
    }

    return buff;
}
