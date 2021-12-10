use serde::Deserialize;
use std::io;
use std::collections::HashMap;
use std::env;

#[derive(Deserialize, Debug)]
pub struct Channel {
    id: u32,
    name: String,
    description: String
}

impl Channel {
    pub fn insert(user_hash: String, name: String, description: String) {
        //TODO call API to add channel
    }

    pub fn close() {

    }
}

pub fn display_main_menu(api_host: String, api_port: String) -> u32 {
    let mut answer = String::from("1");

    while answer.ne("0") {
        std::process::Command::new("clear").status().unwrap();

        println!("========================\n       Main Menu       \n========================");
        println!("1 : Open");
        println!("2 : New");
        println!("0 : Exit");

        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        match &*answer {
            "1" => {
                display_channel_menu(api_host.clone(), api_port.clone());
            },
            "2" => {
                println!("Coming soon...");
                display_main_menu(api_host.clone(), api_port.clone());
                break;
            },
            _ => (),
        }
    }

    0
}

fn display_channel_menu(api_host: String, api_port: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut answer = String::from("1");
    let mut post_data = HashMap::new();

    post_data.insert("token", env::var("token").unwrap());
    post_data.insert("action", String::from("get"));
    post_data.insert("id", String::from("all"));

    //TODO add status if I can not hit URL
    let client = reqwest::blocking::Client::new();
    let res = client.post("http://".to_owned() + &*api_host + ":" + &*api_port + "/api/channel")
        .json(&post_data)
        .send()?
        .json::<HashMap<String, String>>()?;

    let mut channels: Vec<Channel> = Vec::new();

    match res.get("channels") {
        Some(c) => {
            channels = serde_json::from_str(c).unwrap();
        },
        _ => ()
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

        println!("answer = {}", answer);

        for channel in &channels {
            if channel.id.to_string() == answer.to_string() {
                println!("{} {} {}", channel.id, channel.name, channel.description);
            }
        }
        //channels.iter().map(|channel| println!("{} {} {}", channel.id, channel.name, channel.description));
        //channels.into_iter().filter(|channel| channel.id == 20);
        /*for channel in channels.borrow() {
            //println!("{} {}", channel.id, channel.name);

            if channel.id.to_string() == answer.to_string() {
                println!("{} {} {}", channel.id, channel.name, channel.description);
            }
        }*/
    }

    Ok(())
}