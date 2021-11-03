extern crate ini;
use ini::Ini;
use std::fs;
use std::path::Path;
use std::process;
use std::{thread, time};

static CONFIG_FILE: &str = "config/config.ini";

pub struct Config {
    pub domain: String,
    pub port_dest: u16,
    pub logs_directory: String,
}

pub fn check_init_file() -> Config {
    if Path::new(CONFIG_FILE).exists() == true {
        return parse_init_file();
    } else {
        create_new_config_file();
        let config = Config {
            domain: "127.0.0.1".to_string(),
            port_dest: 6969,
            logs_directory: "logs".to_string(),
        };
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
        .set("domain", "127.0.0.1")
        .set("portDest", "6969");
    conf.with_section(Some("LOGS SETTINGS"))
        .set("directory", "logs");
    conf.write_to_file(CONFIG_FILE).unwrap();
}
