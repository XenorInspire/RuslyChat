extern crate ini;

use crate::init;
use crate::log;

use ini::Ini;
use init::Config;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rpassword::read_password;
use rsa::{pkcs1::ToRsaPublicKey, RsaPublicKey};
use std::collections::HashMap;
use std::env;
use std::io;

// Get credentials of the API connection
pub fn request_login(config: Config, pub_key: RsaPublicKey) -> u8 {
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

    std::process::Command::new("clear").status().unwrap();

    match api_login(
        config.domain.clone(),
        config.port_dest.clone().to_string(),
        login,
        password,
        pub_key.clone(),
    ) {
        1 => {
            println!("Connection refused !");
            log::get_logger().log(
                "Connection refused ! Check your credentials".to_string(),
                log::LogLevel::ERROR,
            );
            return 1;
        }

        2 => {
            println!("Connection failed ! Check your internet connection");
            log::get_logger().log(
                "Connection failed ! Check your internet connection".to_string(),
                log::LogLevel::ERROR,
            );
            return 2;
        }

        0 => {
            return 0;
        }
        _ => unreachable!(),
    };
}

// Authenticate the user to the API
pub fn api_login(
    api_host: String,
    api_port: String,
    login: String,
    password: String,
    public_key: RsaPublicKey,
) -> u8 {
    let mut post_data = HashMap::new();

    post_data.insert("login", login);
    post_data.insert("password", password);
    post_data.insert(
        "public_key",
        ToRsaPublicKey::to_pkcs1_pem(&public_key).unwrap(),
    );

    let client = reqwest::blocking::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let res = client
        .post("https://".to_owned() + &*api_host + ":" + &*api_port + "/api/login")
        .json(&post_data)
        .send();

    let res = match res {
        Ok(result) => result,
        Err(e) => {
            println!("The RuslyChat server isn't reachable :(");
            log::get_logger().log(e.to_string(), log::LogLevel::ERROR);
            return 2;
        }
    };

    let res = res.json::<HashMap<String, String>>();

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
        let mut api_public_key = String::new();

        match res.get("token") {
            Some(t) => token = t.clone(),
            _ => (),
        }

        match res.get("public_key") {
            Some(p) => api_public_key = p.clone(),
            _ => (),
        }

        let mut conf = Ini::load_from_file(init::CONFIG_FILE).unwrap();
        conf.with_section(Some("NETWORK SETTINGS"))
            .set("public_key", api_public_key);
        conf.write_to_file(init::CONFIG_FILE).unwrap();

        env::set_var("TOKEN", token);
        std::process::Command::new("clear").status().unwrap();
        return 0;
    } else {
        std::process::Command::new("clear").status().unwrap();
        return 1;
    }
}
