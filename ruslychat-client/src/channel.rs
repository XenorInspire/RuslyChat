use crate::log;
use crate::message;
use log::{get_logger, LogLevel};
use rand::rngs::OsRng;
use rsa::RsaPrivateKey;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io;

#[derive(Deserialize, Debug)]
pub struct Channel {
    id: u32,
    name: String,
    description: String,
}

// Main menu of RuslyChat when you are connected
pub fn display_main_menu(api_host: String, api_port: String, priv_key: RsaPrivateKey, rng: OsRng) {
    let mut answer = String::from("1");

    while answer.ne("0") {
        println!("========================\n       Main Menu       \n========================");
        println!("1 : Open a channel");
        println!("2 : Create a new channel");
        println!("0 : Exit");

        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        match &*answer {
            "1" => {
                if display_channel_menu(
                    api_host.clone(),
                    api_port.clone(),
                    priv_key.clone(),
                    rng.clone(),
                ) == 1
                {
                    println!("Connection failed! Can't get list of channels");
                }
                std::process::Command::new("clear").status().unwrap();
            }
            "2" => {
                create_channel_menu(api_host.clone(), api_port.clone());
            }
            _ => {
                std::process::Command::new("clear").status().unwrap();
            }
        }
    }

    std::process::Command::new("clear").status().unwrap();
}

// Get the list of all the channels the user has access and display it
fn display_channel_menu(
    api_host: String,
    api_port: String,
    priv_key: RsaPrivateKey,
    rng: OsRng,
) -> u8 {
    let mut answer = String::from("1");
    let mut post_data = HashMap::new();

    post_data.insert("token", env::var("TOKEN").unwrap());
    post_data.insert("action", String::from("get"));
    post_data.insert("id", String::from("all"));

    let client = reqwest::blocking::Client::new();
    let res = client
        .post("http://".to_owned() + &*api_host + ":" + &*api_port + "/api/channel")
        .json(&post_data)
        .send();

    let res = match res {
        Ok(result) => result,
        Err(_) => {
            log::get_logger().log(
                "The RuslyChat server isn't reachable :(".to_string(),
                log::LogLevel::ERROR,
            );
            return 1;
        }
    };

    let res = res.json::<HashMap<String, String>>();

    let res = match res {
        Ok(hash) => hash,
        Err(_) => {
            log::get_logger().log(
                "Connection failed! Can't get list of channels".to_string(),
                log::LogLevel::ERROR,
            );
            return 1;
        }
    };

    let mut channels: Vec<Channel> = Vec::new();

    match res.get("channels") {
        Some(c) => channels = serde_json::from_str(c).unwrap(),
        _ => (),
    }

    while answer.ne("0") {
        let mut buff = String::new();

        std::process::Command::new("clear").status().unwrap();

        println!("========================\n     Channel List      \n========================");
        println!("0 : Exit");
        for channel in &channels {
            println!("{} : {}", channel.id, channel.name);
        }

        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        for channel in &channels {
            if channel.id.to_string() == answer.to_string() {
                display_channel(channel.name.clone(), channel.description.clone());
                let res = message::chat(
                    channel.id.to_string(),
                    api_host.clone(),
                    api_port.clone(),
                    priv_key.clone(),
                    rng.clone(),
                );

                if res != 0 {
                    answer = "0".to_string();
                }
            }
        }
    }

    return 0;
}

// Get all the info about one channel and display it
fn display_channel(name: String, description: String) -> u8 {
    let mut buff_enter = String::new();

    std::process::Command::new("clear").status().unwrap();

    println!("========================");
    println!("NAME:\n{}", name);
    println!("           =-=          ");
    println!("DESCRIPTION:\n{}", description);
    println!("========================");
    println!("Press enter to load previous messages.");

    io::stdin()
        .read_line(&mut buff_enter)
        .expect("Reading from stdin failed");

    return 0;
}

// Process for channel creation
fn create_channel_menu(api_host: String, api_port: String) {
    let mut name = String::from("0");
    let mut description = String::from("0");

    while name.eq("0") || description.eq("0") {
        std::process::Command::new("clear").status().unwrap();

        println!("========================\n Channel Creation Menu \n========================");
        println!("Channel name:");

        let mut buff_name = String::new();
        io::stdin()
            .read_line(&mut buff_name)
            .expect("Reading from stdin failed");
        name = buff_name.trim().to_string();

        println!("Channel description (can be empty):");

        let mut buff_description = String::new();
        io::stdin()
            .read_line(&mut buff_description)
            .expect("Reading from stdin failed");
        description = buff_description.trim().to_string();
    }

    create_channel(name, description, api_host, api_port);
}

// Send all the info from the user to create the channel (API side with BDD)
fn create_channel(name: String, description: String, api_host: String, api_port: String) -> u8 {
    let mut post_data = HashMap::new();

    post_data.insert("token", env::var("TOKEN").unwrap());
    post_data.insert("action", String::from("set"));
    post_data.insert("name", name);
    post_data.insert("description", description);

    let client = reqwest::blocking::Client::new();
    let res = client
        .post("http://".to_owned() + &*api_host + ":" + &*api_port + "/api/channel")
        .json(&post_data)
        .send();

    let res = match res {
        Ok(result) => result,
        Err(_) => {
            log::get_logger().log(
                "The RuslyChat server isn't reachable :(".to_string(),
                log::LogLevel::ERROR,
            );
            return 2;
        }
    };

    let res = res.json::<HashMap<String, String>>();

    let res = match res {
        Ok(hash) => hash,
        Err(_) => {
            log::get_logger().log(
                "Connection failed! Can't create a new channel".to_string(),
                log::LogLevel::FATAL,
            );
            println!("Connection failed! Check your internet connection");
            return 2;
        }
    };

    let mut channel_creation_status = String::new();

    match res.get("channel") {
        Some(m) => channel_creation_status = m.clone(),
        _ => (),
    }

    if channel_creation_status.eq("OK") {
        std::process::Command::new("clear").status().unwrap();
        println!("Channel created!");
    } else {
        std::process::Command::new("clear").status().unwrap();
        get_logger().log("Channel creation error...".to_string(), LogLevel::ERROR);
        return 1;
    }

    return 0;
}
