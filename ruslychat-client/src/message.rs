use crate::log;
use crate::message;
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug)]
pub struct Message {
    pub id: u32,
    pub content: String,
    pub date: String,
}

pub fn chat(last_message_id: u32, channel_id: String, api_host: String, api_port: String) {
    let (tx, rx) = mpsc::channel();
    let channel_id_cpy = channel_id.clone();
    let api_host_cpy = api_host.clone();
    let api_port_cpy = api_port.clone();

    let _thread = thread::spawn(move || {
        //let channel_id_to_send = channel_id_cpy.clone();
        let mut last_message_id_to_send = last_message_id.clone();
        
        loop {
            let res = receive_message(
                channel_id_cpy.clone(),
                last_message_id_to_send,
                api_host_cpy.clone(),
                api_port_cpy.clone(),
            );

            if res > 0 {
                last_message_id_to_send = res;
            }

            thread::sleep(Duration::from_millis(5000));

            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    });

    /*
    let mut line = String::new();
    let stdin = io::stdin();
    let _ = stdin.read_line(&mut line);
    */

    let mut answer: String = String::from("0");
    let mut rng = OsRng;
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    while answer.ne("!exit") {
        let mut buff_chat = String::new();

        io::stdin()
            .read_line(&mut buff_chat)
            .expect("Reading from stdin failed");
        answer = buff_chat.trim().to_string();

        if answer.chars().next().expect("0").to_string() == "!".to_string() {
            check_command(answer.clone());
        } else {
            send_message(
                answer.clone(),
                channel_id.clone(),
                rng,
                pub_key.clone(),
                api_host.clone(),
                api_port.clone(),
            );
        }
    }

    let _ = tx.send(());
}

fn send_message(
    content: String,
    channel_id: String,
    rng: OsRng,
    pub_key: RsaPublicKey,
    api_host: String,
    api_port: String,
) -> u8 {
    let mut post_data = HashMap::new();
    let now: DateTime<Utc> = Utc::now();
    let time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let new_content = encrypt_message(&*content, rng, pub_key);
    let encrypted_content = serde_json::to_string(&new_content).unwrap();
    println!("{}", encrypted_content.clone());
    post_data.insert("token", env::var("TOKEN").unwrap());
    post_data.insert("action", String::from("set"));
    post_data.insert("id", channel_id);
    post_data.insert("content", encrypted_content);
    post_data.insert("date", time);

    //TODO add status if I can not hit URL
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("http://".to_owned() + &*api_host + ":" + &*api_port + "/api/message")
        .json(&post_data)
        .send()
        .expect("Connection failed!")
        .json::<HashMap<String, String>>();

    let res = match res {
        Ok(hash) => hash,
        Err(_) => {
            log::get_logger().log(
                "Connection failed! Can not create a new message".to_string(),
                log::LogLevel::FATAL,
            );
            println!("Connection failed! Check your internet connection");
            return 2;
        }
    };

    let mut message_creation_status = String::new();

    match res.get("message") {
        Some(m) => message_creation_status = m.clone(),
        _ => (),
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

fn receive_message(
    channel_id: String,
    min_message_id: u32,
    api_host: String,
    api_port: String,
) -> u32 {
    let mut post_data = HashMap::new();

    post_data.insert("token", env::var("TOKEN").unwrap());
    post_data.insert("action", String::from("get"));
    post_data.insert("channel_id", channel_id);
    post_data.insert("min_message_id", min_message_id.to_string());
    post_data.insert("count", String::from("20"));

    //TODO add status if I can not hit URL
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("http://".to_owned() + &*api_host + ":" + &*api_port + "/api/message")
        .json(&post_data)
        .send()
        .expect("Connection failed!")
        .json::<HashMap<String, String>>();

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

    let mut messages: Vec<message::Message> = Vec::new();
    let mut last_message_id: u32 = 0;

    match res.get("messages") {
        Some(m) => messages = serde_json::from_str(m).unwrap(),
        _ => (),
    }

    for message in &messages {
        last_message_id = message.id;
        println!("[{}] : {}", message.date, message.content);
    }

    last_message_id
}

fn encrypt_message(
    message: &str,
    mut rng: rand::rngs::OsRng,
    pub_key: rsa::RsaPublicKey,
) -> std::vec::Vec<u8> {
    let mut message_encrypted = pub_key
        .encrypt(
            &mut rng,
            PaddingScheme::new_pkcs1v15_encrypt(),
            &message.as_bytes(),
        )
        .expect("failed to encrypt");

    log::get_logger().log(
        format!("{:?}", message_encrypted.clone()),
        log::LogLevel::DEBUG,
    );

    //message_encrypted.retain(|&x| x != 255);
    //message_encrypted = vec![120, 12, 32, 18];
    // let test = String::from_utf8_lossy(&message_encrypted);

    // log::get_logger().log(format!("{:?}", test.clone()), log::LogLevel::DEBUG);

    /*String::from_utf8(
        ,
    )
    .expect("Found invalid UTF-8")*/

    // "test".to_string()
    message_encrypted
}

fn decrypt_message(message: std::vec::Vec<u8>, priv_key: rsa::RsaPrivateKey) -> String {
    String::from_utf8(
        priv_key
            .decrypt(PaddingScheme::new_pkcs1v15_encrypt(), &message)
            .expect("failed to decrypt"),
    )
    .unwrap()
}

fn check_command(command: String) {
    match command.as_str() {
        "!help" => display_help(),
        _ => unreachable!(),
    }
}

fn display_help() {
    println!("List of commands :");
    println!("!help => Display this help menu");
    println!("!exit => Exit the channel");
    println!("!add <user> => Add a user to this channel");
    println!("!delete => Delete the conversation\nThis command will kick all the members.");
}
