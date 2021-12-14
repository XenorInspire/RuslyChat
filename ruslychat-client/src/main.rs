extern crate chrono;
extern crate ini;
extern crate rand;
extern crate rpassword;
extern crate rsa;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::env;
use std::io;

mod channel;
mod init;
mod log;
mod login;
mod message;

fn main() {
    let mut config = init::check_init_file();
    let mut backup = config.clone();
    let mut answer = String::from("1");
    env::set_var("PATH_LOGGER", config.logs_directory.clone());
    log::get_logger().log("Ruslychat is starting".to_string(), log::LogLevel::INFO);

    let mut rng = OsRng;
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    log::get_logger().log("Ruslychat started!".to_string(), log::LogLevel::INFO);
    std::process::Command::new("clear").status().unwrap();
    
    // Display main menu
    while answer.eq("0") == false {
        println!("========================\n Welcome to RuslyChat !\n========================");
        println!("1 : Log in");
        println!("2 : Manage settings");
        println!("0 : Exit");

        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        match &*answer {
            // Log in process
            "1" => {
                if login::request_login(config.clone(), pub_key.clone()) == 0 {
                    channel::display_main_menu(
                        config.domain.clone(),
                        config.port_dest.clone().to_string(),
                        priv_key.clone(),
                        rng.clone(),
                    )
                }
            }

            // Settings management process
            "2" => {
                config = init::change_config_values(config);
                if config != backup {
                    init::create_new_config_file(init::CURRENT_CONFIG_FILE_MODE, config.clone());
                    backup = config.clone();
                }
            }
            _ => (),
        }
    }
}
