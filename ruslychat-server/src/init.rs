extern crate ini;
use ini::Ini;
use std::fs;
use std::path::Path;
use std::process;
use std::{thread, time};

static CONFIG_FILE: &str = "config/config.ini";

pub struct Config {
    pub port: u16,
    pub logs_directory: String,
    pub database: String,
    pub user: String,
    pub passwd: String,
}

pub fn check_init_file() -> Config {
    if Path::new(CONFIG_FILE).exists() == true {
        return parse_init_file();
    } else {
        create_new_config_file();
        let config = Config {
            port: 6969,
            logs_directory: "logs".to_string(),
            database: "rusly_db".to_string(),
            user: "rusly".to_string(),
            passwd: "root".to_string(),
        };
        return config;
    }
}

fn parse_init_file() -> Config {
    let conf = Ini::load_from_file(CONFIG_FILE).unwrap();
    let network_section = conf.section(Some("NETWORK SETTINGS")).unwrap();
    let port = network_section.get("port").unwrap();

    let logs_section = conf.section(Some("LOGS SETTINGS")).unwrap();
    let logs_directory = logs_section.get("directory").unwrap();

    let database_section = conf.section(Some("DATABASE SETTINGS")).unwrap();
    let database = database_section.get("database").unwrap();
    let user = database_section.get("user").unwrap();
    let passwd = database_section.get("passwd").unwrap();

    let config = Config {
        port: port.parse::<u16>().unwrap(),
        logs_directory: logs_directory.to_string(),
        database: database.to_string(),
        user: user.to_string(),
        passwd: passwd.to_string(),
    };

    return config;
}

fn create_new_config_file() {
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

    let mut conf = Ini::new();
    conf.with_section(Some("NETWORK SETTINGS"))
        .set("port", "6969");
    conf.with_section(Some("LOGS SETTINGS"))
        .set("directory", "logs");
    conf.with_section(Some("DATABASE SETTINGS"))
        .set("database", "rusly_db")
        .set("user", "rusly")
        .set("passwd", "root");
    conf.write_to_file(CONFIG_FILE).unwrap();
}
