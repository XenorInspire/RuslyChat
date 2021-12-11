extern crate chrono;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::io;

mod init;
mod connect_tcp;
mod channel;
mod user;
mod log;
mod message;

fn main() {
    let mut config = init::check_init_file();
    let mut backup = config.clone();
    let mut answer = String::from("1");

    std::process::Command::new("clear").status().unwrap();

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
            "1" => {
                user::request_login(config.clone());
            },
            "2" => {
                config = init::change_config_values(config);
                if config != backup {
                    init::create_new_config_file(init::CURRENT_CONFIG_FILE_MODE, config.clone());
                    backup = config.clone();
                }
            },
            _ => (),
        }
    }
}
