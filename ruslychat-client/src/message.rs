use crate::init;
use crate::log;
use crate::user;
use chrono::{DateTime, Utc};
use ini::Ini;
use rand::rngs::OsRng;
use rsa::{pkcs1::FromRsaPublicKey, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use user::User;

#[derive(Deserialize, Debug)]
struct Message {
    id: u32,
    content: Vec<u8>,
    date: String,
    username: String,
}

// Main function of RuslyChat chat
pub fn chat(
    channel_id: String,
    api_host: String,
    api_port: String,
    priv_key: RsaPrivateKey,
    rng: OsRng,
) {
    println!("!help to get available commands\n");
    let mut answer: String = String::from("0");

    let (tx, rx) = mpsc::channel();
    let channel_id_cpy = channel_id.clone();
    let api_host_cpy = api_host.clone();
    let api_port_cpy = api_port.clone();
    let priv_key_cpy = priv_key.clone();

    // Get the last messages from the API
    let _thread = thread::spawn(move || {
        let mut last_message_id_to_send = 0;
        loop {
            let res = receive_message(
                channel_id_cpy.clone(),
                last_message_id_to_send,
                priv_key_cpy.clone(),
                api_host_cpy.clone(),
                api_port_cpy.clone(),
            );

            if res > 0 {
                last_message_id_to_send = res;
            }

            thread::sleep(Duration::from_millis(2500));

            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }
        }
    });

    while answer.ne("!exit") && answer.ne("!delete") {
        let mut buff_chat = String::new();

        io::stdin()
            .read_line(&mut buff_chat)
            .expect("Reading from stdin failed");
        answer = buff_chat.trim().to_string();

        if answer.len() > 0 {
            if answer.len() <= 250 {
                if answer.chars().next().expect("0").to_string() == "!".to_string() {
                    check_command(
                        api_host.clone(),
                        api_port.clone(),
                        channel_id.clone(),
                        answer.clone(),
                    );
                } else {
                    let res_send_message = send_message(
                        answer.clone(),
                        channel_id.clone(),
                        rng,
                        api_host.clone(),
                        api_port.clone(),
                    );

                    if res_send_message == 2 {
                        break;
                    }
                }
            } else {
                println!("Message too long (250 characters max)");
            }
        }
    }

    let _ = tx.send(());
}

// Encrypt and send the message to the API
fn send_message(
    content: String,
    channel_id: String,
    rng: OsRng,
    api_host: String,
    api_port: String,
) -> u8 {
    let mut post_data = HashMap::new();
    let now: DateTime<Utc> = Utc::now();
    let time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let conf = Ini::load_from_file(init::CONFIG_FILE).unwrap();
    let network_section = conf.section(Some("NETWORK SETTINGS")).unwrap();
    let public_key_string = network_section.get("public_key").unwrap();
    let api_public_key = RsaPublicKey::from_pkcs1_pem(&public_key_string).unwrap();
    let new_content = encrypt_message(&*content, rng, api_public_key);
    let encrypted_content = serde_json::to_string(&new_content).unwrap();
    post_data.insert("token", env::var("TOKEN").unwrap());
    post_data.insert("action", String::from("set"));
    post_data.insert("id", channel_id);
    post_data.insert("content", encrypted_content);
    post_data.insert("date", time);

    let client = reqwest::blocking::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let res = client
        .post("https://".to_owned() + &*api_host + ":" + &*api_port + "/api/message")
        .json(&post_data)
        .send();

    let res = match res {
        Ok(result) => result,
        Err(e) => {
            println!("The RuslyChat server isn't reachable :(");
            log::get_logger().log(e.to_string(), log::LogLevel::ERROR);
            println!("Connection failed! Check your internet connection");
            return 1;
        }
    };

    let res = res.json::<HashMap<String, String>>();

    let res = match res {
        Ok(hash) => hash,
        Err(_) => {
            log::get_logger().log(
                "Connection failed! Can not create a new message".to_string(),
                log::LogLevel::FATAL,
            );
            println!("Connection failed! Check your internet connection");
            return 1;
        }
    };

    let mut message_creation_status = String::new();

    match res.get("message") {
        Some(m) => message_creation_status = m.clone(),
        _ => (),
    }

    let mut channel_status = String::new();

    match res.get("channel") {
        Some(c) => channel_status = c.clone(),
        _ => (),
    }

    if channel_status.ne("OK") {
        log::get_logger().log("Channel deleted!".to_string(), log::LogLevel::ERROR);
        return 2;
    }

    if message_creation_status.ne("OK") {
        log::get_logger().log(
            "Message creation error...".to_string(),
            log::LogLevel::ERROR,
        );
        return 1;
    }

    return 0;
}

// Decrypt and get the message from the API
fn receive_message(
    channel_id: String,
    min_message_id: u32,
    priv_key: RsaPrivateKey,
    api_host: String,
    api_port: String,
) -> u32 {
    let mut post_data = HashMap::new();

    post_data.insert("token", env::var("TOKEN").unwrap());
    post_data.insert("action", String::from("get"));
    post_data.insert("channel_id", channel_id);
    post_data.insert("min_message_id", min_message_id.to_string());
    post_data.insert("count", String::from("20"));

    let client = reqwest::blocking::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let res = client
        .post("https://".to_owned() + &*api_host + ":" + &*api_port + "/api/message")
        .json(&post_data)
        .send();

    let res = match res {
        Ok(result) => result,
        Err(e) => {
            println!("The RuslyChat server isn't reachable :(");
            log::get_logger().log(e.to_string(), log::LogLevel::ERROR);
            println!("Connection failed! Check your internet connection");

            return 0;
        }
    };

    let res = res.json::<HashMap<String, String>>();

    let res = match res {
        Ok(hash) => hash,
        Err(_) => {
            log::get_logger().log(
                "Connection failed! Can not get any message".to_string(),
                log::LogLevel::FATAL,
            );
            println!("Connection failed! Check your internet connection");
            return 0;
        }
    };

    let mut messages: Vec<Message> = Vec::new();
    let mut last_message_id: u32 = 0;

    match res.get("messages") {
        Some(m) => messages = serde_json::from_str(m).unwrap(),
        _ => (),
    }

    for message in &messages {
        let message_content = decrypt_message(message.content.clone(), priv_key.clone());

        last_message_id = message.id;
        println!(
            "[{}][{}] : {}",
            message.date, message.username, message_content
        );
    }

    last_message_id
}

// Encrypt a message with the public key
pub fn encrypt_message(
    message: &str,
    mut rng: rand::rngs::OsRng,
    pub_key: rsa::RsaPublicKey,
) -> std::vec::Vec<u8> {
    let message_encrypted = pub_key
        .encrypt(
            &mut rng,
            PaddingScheme::new_pkcs1v15_encrypt(),
            &message.as_bytes(),
        )
        .expect("failed to encrypt");

    message_encrypted
}

// Decrypt a message with the private key
pub fn decrypt_message(message: std::vec::Vec<u8>, priv_key: rsa::RsaPrivateKey) -> String {
    String::from_utf8(
        priv_key
            .decrypt(PaddingScheme::new_pkcs1v15_encrypt(), &message)
            .expect("failed to decrypt"),
    )
    .unwrap()
}

/*********                      COMMAND PART                        ***********/

fn check_command(api_host: String, api_port: String, channel_id: String, command: String) {
    let split = command.split(" ");

    let args: Vec<&str> = split.collect();
    let command_name = args[0];

    match command_name {
        "!help" => command_help(),
        "!exit" => (),
        "!add" => command_add(args, channel_id, api_host, api_port),
        "!list" => command_list(channel_id, api_host, api_port),
        "!delete" => command_delete(channel_id, api_host, api_port),
        _ => println!("Invalid command!"),
    }
}

// Display help menu with all the commands
fn command_help() {
    println!("[I]           List of commands            [I]");
    println!("!help         => Display this help menu");
    println!("!exit         => Exit the channel");
    println!("!add <user>   => Add a user to this channel");
    println!("!list         => List all users in this channel");
    println!("!delete       => Delete the conversation");
    println!("                      -\n");
}

// This function permits to add a user to the current channel
fn command_add(args: Vec<&str>, channel_id: String, api_host: String, api_port: String) {
    if args.len() < 2 {
        println!("Usage: !add <user>");
    } else {
        let mut post_data = HashMap::new();

        post_data.insert("token", env::var("TOKEN").unwrap());
        post_data.insert("action", String::from("add_user"));
        post_data.insert("login", String::from(args[1]));
        post_data.insert("channel_id", channel_id);

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
                return;
            }
        };

        let res = res.json::<HashMap<String, String>>();

        let res = match res {
            Ok(hash) => hash,
            Err(_) => {
                log::get_logger().log(
                    "Connection failed! Can not delete the channel".to_string(),
                    log::LogLevel::FATAL,
                );
                println!("Connection failed! Check your internet connection");
                return;
            }
        };

        let mut user_add_status = String::new();

        match res.get("channel") {
            Some(c) => user_add_status = c.clone(),
            _ => (),
        }

        if user_add_status.eq("OK") {
            println!("User added!");
        } else {
            println!("{}", user_add_status);
            log::get_logger().log(
                "Error on adding user to channel...".to_string(),
                log::LogLevel::ERROR,
            );
        }
    }
}

// This function permits to add a user to the current channel
fn command_list(channel_id: String, api_host: String, api_port: String) {
    let mut post_data = HashMap::new();

    post_data.insert("token", env::var("TOKEN").unwrap());
    post_data.insert("action", String::from("get_users"));
    post_data.insert("channel_id", channel_id);

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
            return;
        }
    };

    let res = res.json::<HashMap<String, String>>();

    let res = match res {
        Ok(hash) => hash,
        Err(_) => {
            log::get_logger().log(
                "Connection failed! Can not delete the channel".to_string(),
                log::LogLevel::FATAL,
            );
            println!("Connection failed! Check your internet connection");
            return;
        }
    };

    let mut users: Vec<User> = Vec::new();

    match res.get("users") {
        Some(u) => users = serde_json::from_str(u).unwrap(),
        _ => (),
    }

    println!("--------------------------------------");
    for user in &users {
        println!("{} : {}", user.username, user.email);
    }
    println!("--------------------------------------");
}

// This function permits to delete the current channel
fn command_delete(channel_id: String, api_host: String, api_port: String) {
    let mut post_data = HashMap::new();

    post_data.insert("token", env::var("TOKEN").unwrap());
    post_data.insert("action", String::from("del"));
    post_data.insert("channel_id", channel_id);

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
            return;
        }
    };

    let res = res.json::<HashMap<String, String>>();

    let res = match res {
        Ok(hash) => hash,
        Err(_) => {
            log::get_logger().log(
                "Connection failed! Can not delete the channel".to_string(),
                log::LogLevel::FATAL,
            );
            println!("Connection failed! Check your internet connection");
            return;
        }
    };

    let mut channel_creation_status = String::new();

    match res.get("channel") {
        Some(c) => channel_creation_status = c.clone(),
        _ => (),
    }

    if channel_creation_status.eq("OK") {
        std::process::Command::new("clear").status().unwrap();
        println!("Channel deleted!");
    } else {
        std::process::Command::new("clear").status().unwrap();
        log::get_logger().log(
            "Channel deletion error...".to_string(),
            log::LogLevel::ERROR,
        );
    }
}
