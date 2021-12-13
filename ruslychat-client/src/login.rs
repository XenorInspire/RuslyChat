use crate::channel;
use crate::init;
use crate::log;

use init::Config;
use rpassword::read_password;
use std::collections::HashMap;
use std::env;
use std::io;

pub fn request_login(config: Config) {
    let mut login = String::from("0");
    let mut password = String::from("0");

    while login.eq("0") || password.eq("0") {
        let mut buff_login = String::new();

        std::process::Command::new("clear").status().unwrap();

        println!("Login:");

        io::stdin()
            .read_line(&mut buff_login)
            .expect("Reading from stdin failed");
        login = buff_login.trim().to_string();

        println!("Password:");

        password = read_password().unwrap();
    }

    //TODO change api_port to config when available
    match api_login(
        config.domain.clone(),
        config.port_dest.clone().to_string(),
        login,
        password,
    ) {
        1 => {
            println!("Connection refused !");
            log::get_logger().log(
                "Connection refused ! Check your credentials".to_string(),
                log::LogLevel::ERROR,
            );
        }

        2 => {
            println!("Connection failed ! Check your internet connection");
            log::get_logger().log(
                "Connection failed ! Check your internet connection".to_string(),
                log::LogLevel::ERROR,
            );
        }

        0 => {
            channel::display_main_menu(config.domain.clone(), config.port_dest.clone().to_string())
        }
        _ => unreachable!(),
    };
}

fn api_login(api_host: String, api_port: String, login: String, password: String) -> u8 {
    let mut post_data = HashMap::new();

    post_data.insert("login", login);
    post_data.insert("password", password);

    //TODO add status if I can not hit URL
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("http://".to_owned() + &*api_host + ":" + &*api_port + "/api/login")
        .json(&post_data)
        .send()
        .expect("Connection failed!")
        .json::<HashMap<String, String>>();

    let res = match res {
        Ok(hash) => hash,
        Err(_) => {
            log::get_logger().log("Connection failed!".to_string(), log::LogLevel::FATAL);
            println!("Connection failed! Check your internet connection");
            return 2;
        }
    };

    let mut connection = String::new();

    match res.get("connection") {
        Some(c) => connection = c.clone(),
        _ => (),
    }

    println!("connection: {:?}", connection);

    if connection == "Success" {
        let mut token = String::new();

        match res.get("token") {
            Some(t) => token = t.clone(),
            _ => (),
        }

        // println!("token: {:?}", token.clone());
        env::set_var("TOKEN", token);
        std::process::Command::new("clear").status().unwrap();
        return 0;
    } else {
        //std::process::Command::new("clear").status().unwrap();
        return 1;
    }
}
