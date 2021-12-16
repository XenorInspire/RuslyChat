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

// Implement the clone function to the struct channel
impl Clone for Channel {
    fn clone(&self) -> Self {
        Channel {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}

// Main menu of RuslyChat when you are connected
pub fn display_main_menu(api_host: String, api_port: String, priv_key: RsaPrivateKey, rng: OsRng) {
    let mut answer = String::from("1");

    while answer.ne("0") {
        println!("========================\n       Main Menu       \n========================");
        println!("1 : Open a chat");
        println!("2 : Create a new chat");
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
                    println!("Connection failed! Can't get list of chats");
                }
                print!("\x1B[2J\x1B[1;1H");
            }
            "2" => {
                create_channel_menu(api_host.clone(), api_port.clone());
            }
            _ => {
                print!("\x1B[2J\x1B[1;1H");
            }
        }
    }

    print!("\x1B[2J\x1B[1;1H");
}

// Get the list of all the channels the user has access and display it
fn display_channel_menu(
    api_host: String,
    api_port: String,
    priv_key: RsaPrivateKey,
    rng: OsRng,
) -> u8 {
    let mut answer = String::from("1");
    while answer.ne("0") {
        let mut post_data = HashMap::new();

        post_data.insert("token", env::var("TOKEN").unwrap());
        post_data.insert("action", String::from("get"));
        post_data.insert("id", String::from("all"));

        let client = reqwest::blocking::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let res = client
            .post("https://".to_owned() + &*api_host + ":" + &*api_port + "/api/channel")
            .json(&post_data)
            .send();

        let res = match res {
            Ok(result) => result,
            Err(e) => {
                println!("The RuslyChat server isn't reachable :(");
                log::get_logger().log(e.to_string(), log::LogLevel::ERROR);
                return 1;
            }
        };

        let res = res.json::<HashMap<String, String>>();

        let res = match res {
            Ok(hash) => hash,
            Err(_) => {
                log::get_logger().log(
                    "Connection failed! Can't get list of chats".to_string(),
                    log::LogLevel::ERROR,
                );
                return 1;
            }
        };

        let mut channels: Vec<Channel> = Vec::new();
        let mut channel_list: HashMap<String, Channel> = HashMap::new();
        let mut i = 1;

        match res.get("channels") {
            Some(c) => channels = serde_json::from_str(c).unwrap(),
            _ => (),
        }

        let mut buff = String::new();

        print!("\x1B[2J\x1B[1;1H");

        println!("========================\n     Chat List      \n========================");
        println!("0 : Exit");
        for channel in &channels {
            channel_list.insert(i.to_string(), channel.clone());
            println!("{} : {}", i, channel.name);
            i += 1;
        }

        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        if channel_list.contains_key(&answer.to_string()) {
            display_channel(
                channel_list.get(&answer.to_string()).unwrap().name.clone(),
                channel_list
                    .get(&answer.to_string())
                    .unwrap()
                    .description
                    .clone(),
            );
            message::chat(
                channel_list
                    .get(&answer.to_string())
                    .unwrap()
                    .id
                    .to_string(),
                api_host.clone(),
                api_port.clone(),
                priv_key.clone(),
                rng.clone(),
            );
        }
    }

    return 0;
}

// Get all the info about one channel and display it
fn display_channel(name: String, description: String) -> u8 {
    let mut buff_enter = String::new();

    print!("\x1B[2J\x1B[1;1H");

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
        print!("\x1B[2J\x1B[1;1H");

        println!("========================\n Chat Creation Menu \n========================");
        println!("Chat name:");

        let mut buff_name = String::new();
        io::stdin()
            .read_line(&mut buff_name)
            .expect("Reading from stdin failed");
        name = buff_name.trim().to_string();

        println!("Chat description (can be empty):");
        let mut buff_description = String::from("");

        io::stdin()
            .read_line(&mut buff_description)
            .expect("Reading from stdin failed");
        description = buff_description.trim().to_string();

        while buff_description.len() > 255 {
            println!("Description too long! (255 max)");
            io::stdin()
                .read_line(&mut buff_description)
                .expect("Reading from stdin failed");
            description = buff_description.trim().to_string();
        }
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

    let client = reqwest::blocking::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let res = client
        .post("https://".to_owned() + &*api_host + ":" + &*api_port + "/api/channel")
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
            log::get_logger().log(
                "Connection failed! Can't create a new chat".to_string(),
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
        print!("\x1B[2J\x1B[1;1H");
        println!("Chat created!");
    } else {
        print!("\x1B[2J\x1B[1;1H");
        get_logger().log("Chat creation error...".to_string(), LogLevel::ERROR);
        return 1;
    }

    return 0;
}
