use crate::channel;
use crate::init;

use rpassword::read_password;
use std::io;
use std::env;
use std::collections::HashMap;
use init::Config;

pub fn request_login(config: Config) {
    let mut login = String::from("0");
    let mut password = String::from("0");

    while login.eq("0") || password.eq("0") {
        let mut buff_login = String::new();
        let mut buff_password = String::new();

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
    api_login(config.domain.clone(), "6970".to_string(), login.clone(), password.clone());
}

fn api_login(api_host: String, api_port: String, login: String, password: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut post_data = HashMap::new();

    post_data.insert("login", login);
    post_data.insert("password", password);

    //TODO add status if I can not hit URL
    let client = reqwest::blocking::Client::new();
    let res = client.post("http://".to_owned() + &*api_host + ":" + &*api_port + "/api/login")
        .json(&post_data)
        .send()?
        .json::<HashMap<String, String>>()?;

    let mut connection = String::new();

    match res.get("connection") {
        Some(c) => connection = c.clone(),
        _ => ()
    }

    println!("connection: {:?}", connection);

    if connection == "Success" {
        let mut token = String::new();

        match res.get("token") {
            Some(t) => token = t.clone(),
            _ => ()
        }

        println!("token: {:?}", token.clone());

        env::set_var("token", token);

        channel::display_main_menu(api_host, api_port);
    } else {
        std::process::Command::new("clear").status().unwrap();
        println!("Connection refused!");
    }

    Ok(())
}

