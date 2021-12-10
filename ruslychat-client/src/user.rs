use crate::channel;
use crate::init;
use std::io;
use std::env;
use std::collections::HashMap;
use init::Config;

pub fn request_login(config: Config) {
    let mut login = String::from("0");
    let mut password = String::from("0");

    while login.eq("0") || password.eq("0") {
        let mut buff = String::new();

        println!("Login:");

        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        login = buff.trim().to_string();

        println!("Password:");

        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        password = buff.trim().to_string();
    }
    //TODO change api_port to config when available

    api_login(config.domain.clone(), "6970".to_string(), login, password);
}

fn api_login(api_host: String, api_port: String, login: String, password: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut post_data = HashMap::new();
    //let mut token = String::new();

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

        channel::display_main_menu();
    } else {
        println!("Connection refused!");
    }

    Ok(())
}

