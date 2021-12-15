extern crate chrono;
extern crate mysql;
extern crate pwhash;
extern crate rand;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate warp;
extern crate ini;

mod init;
mod log;
mod encrypt;

use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::ToRsaPublicKey, pkcs1::FromRsaPublicKey};
use log::{get_logger, LogLevel};
use mysql::prelude::*;
use mysql::*;
use pwhash::sha512_crypt;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::env;
use warp::Filter;

#[derive(Serialize, Debug)]
struct Channel {
    id: u32,
    name: String,
    description: String,
}

#[derive(Serialize, Debug)]
struct Message {
    id: u32,
    content: Vec<u8>,
    date: String,
    username: String,
}

#[derive(Serialize, Debug)]
struct User {
    email: String,
    username: String,
}

#[tokio::main]
async fn main() {
    let config = init::check_init_file();
    env::set_var("PATH_LOGGER_API", config.logs_directory.clone());
    get_logger().log("Ruslychat API is starting...".to_string(), LogLevel::INFO);

    // For login route
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    // For message route
    let mut rng_message_route = rng.clone();
    let private_key_message_route = private_key.clone();

    get_logger().log("Ruslychat API started!".to_string(), LogLevel::INFO);
    
    // URI POST: /api/login
    // with json data : { "login":"username or email", "password":"password", "public_key":"client_public_key" }
    // For first login and generating the token
    let user_login = warp::path!("login")
        .and(warp::post())
        .and(warp::body::json())
        .map( move |request_data: HashMap<String, String>| {
            let message_data = request_data.clone();
            let mut return_data_json: HashMap<_, String> = HashMap::new();
            let mut message_given_public_key = String::new();

            // For sending result from thread
            let (tx, rx) = mpsc::channel();

            let user_public_key = public_key.clone();

            // Checking password in thread
            let _thread = thread::spawn(move || -> Result<()> {
                let config = init::check_init_file();

                let mut user_given_id = String::new();
                match message_data.get("login") {
                    Some(value) => user_given_id = value.to_string(),
                    None => (),
                }

                get_logger().log(format!("given login: {}", user_given_id), LogLevel::DEBUG);

                // Database connection
                let url: String = "mysql://".to_owned()
                    + &*config.user
                    + ":"
                    + &*config.passwd
                    + "@localhost:3306/"
                    + &*config.database;
                let opts: Opts = Opts::from_url(&*url)?;
                let pool: Pool = Pool::new(opts)?;
                let mut conn: PooledConn = pool.get_conn()?;

                // SQL Request
                let req_select_user = conn.prep(
                    "SELECT * FROM `user` WHERE `email` = :email OR `username` = :username ",
                )?;

                // Response
                let res_select_user: Vec<mysql::Row> = conn.exec(
                    &req_select_user,
                    params! {
                        "email" => user_given_id.clone(),
                        "username" => user_given_id,
                    },
                )?;

                // Parsing response
                let mut hash_from_db = String::new();
                let mut id_from_db: u32 = 0;

                //Hashing the password given
                let given_password = request_data.get("password");
                let mut user_password = String::new();
                match given_password {
                    Some(value) => user_password = value.to_string(),
                    None => (),
                }

                let hash_setup = "$6$salt";
                let hashed_given_pwd = sha512_crypt::hash_with(hash_setup, user_password).unwrap();

                for mut row in res_select_user {
                    // Getting hashed from db
                    id_from_db = row.take("id").unwrap();
                    hash_from_db = row.take("password").unwrap();
                }
                // Good password
                if hashed_given_pwd == hash_from_db {
                    // Generating the token
                    let token: String = rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(150)
                        .map(char::from)
                        .collect();

                    // SQL Request
                    let req_update_user_token =
                        conn.prep("UPDATE `user` SET `token` = :token WHERE `id` = :id")?;

                    // Response
                    let _res_update_user_token: Vec<mysql::Row> = conn.exec(
                        &req_update_user_token,
                        params! {
                            "token" => token.clone(),
                            "id" => id_from_db
                        },
                    )?;
                    // ..........................//

                    // Insert public key
                    match message_data.get("public_key"){
                        Some(value) => message_given_public_key = value.to_string(),
                        None => (),
                    }

                    if message_given_public_key.is_empty() {
                        get_logger().log(format!("Not given public_key"), LogLevel::DEBUG);
                        return_data_json.insert("connection", "Refused".to_string());
                        return_data_json.insert("info", "Missing public key.".to_string());
                        tx.send(return_data_json).unwrap();
 
                    } else {
                        // SQL Request
                        let req_update_user_token = conn.prep("UPDATE `user` SET `public_key` = :public_key WHERE `id` = :id")?;

                        let _res_update_user_token: Vec<mysql::Row> = conn.exec(
                            &req_update_user_token,
                            params! {
                                "public_key" => message_given_public_key,
                                "id" => id_from_db
                            },
                        )?;
                        
                        // Response
                        return_data_json.insert("connection", "Success".to_string());
                        return_data_json.insert("token", token);
                        return_data_json.insert("public_key", ToRsaPublicKey::to_pkcs1_pem(&user_public_key).unwrap());

                        tx.send(return_data_json).unwrap();
                    }
                } else {
                    return_data_json.insert("connection", "Refused".to_string());
                    return_data_json.insert("info", "Wrong login or password.".to_string());
                    tx.send(return_data_json).unwrap();
                }
                Ok(())
            });

            // Getting result from tread
            let received = rx.recv().unwrap();

            // Sending final result
            return warp::reply::json(&received);
        });

    
    
    // URI POST: /api/channel
    // with json data : { "token":"u_token", "action":"get", "id":"c_id|all" }
    // with json data : { "token":"u_token", "action":"del", "id":"c_id" }
    // with json data : { "token":"u_token", "action":"set", "name":"c_name", "description":"c_description" }
    // To get channels
    let channel = warp::path!("channel")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request_data: HashMap<String, String>| {
            let channel_data = request_data.clone();
            let mut return_data_json: HashMap<_, String> = HashMap::new();

            // For sending result from thread
            let (tx, rx) = mpsc::channel();

            // Thread
            let _thread = thread::spawn(move || -> Result<()> {
                let config = init::check_init_file();

                // Database connection
                let url: String = "mysql://".to_owned() + &*config.user + ":" + &*config.passwd + "@localhost:3306/" + &*config.database;
                let opts: Opts = Opts::from_url(&*url)?;
                let pool: Pool = Pool::new(opts)?;
                let mut conn: PooledConn = pool.get_conn()?;

                match channel_data.get("action") {
                    Some(action) => {
                        match action.as_ref() {
                            "get" => {
                                get_logger().log(format!("Action: Channel get"), LogLevel::TRACE);

                                let mut channel_given_id = String::new();
                                let mut channel_given_token = String::new();

                                match channel_data.get("id") {
                                    Some(value) => channel_given_id = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given id: {}", channel_given_id), LogLevel::DEBUG);

                                match channel_data.get("token") {
                                    Some(value) => channel_given_token = value.to_string(),
                                    None => (),
                                }

                                let req_select_channel: Statement;
                                let mut res_select_channel: Vec<mysql::Row> = Vec::new();

                                if channel_given_id.eq("all") {
                                    // SQL Request
                                    req_select_channel = conn.prep("SELECT u.token, c.* FROM user_channel uc LEFT JOIN user u ON uc.id_user = u.id LEFT JOIN channel c ON uc.id_channel = c.id WHERE u.token = :u_token")?;

                                    // Response
                                    res_select_channel = conn.exec(
                                        &req_select_channel,
                                        params! {
                                            "u_token" => channel_given_token,
                                        },
                                    )?;
                                } else {
                                    // SQL Request
                                    req_select_channel = conn.prep("SELECT u.token, c.* FROM user_channel uc LEFT JOIN user u ON uc.id_user = u.id LEFT JOIN channel c ON uc.id_channel = c.id WHERE u.token = :u_token AND c.id = :c_id")?;

                                    // Response
                                    res_select_channel = conn.exec(
                                        &req_select_channel,
                                        params! {
                                            "u_token" => channel_given_token,
                                            "c_id" => channel_given_id,
                                        },
                                    )?;
                                }

                                // Parsing response
                                let mut channels: Vec<_> = Vec::new();

                                for mut row in res_select_channel {
                                    // Getting channel from db
                                    let channel = Channel {
                                        id: row.take("id").unwrap(),
                                        name: row.take("name").unwrap(),
                                        description: row.take("description").unwrap()
                                    };

                                    channels.push(channel);
                                }

                                let channels_serialized = serde_json::to_string(&channels).unwrap();

                                get_logger().log(format!("Serialized channels sent"), LogLevel::DEBUG);

                                return_data_json.insert("channels", channels_serialized);
                            },
                            "del" => {
                                get_logger().log(format!("Action: Channel delete"), LogLevel::TRACE);

                                let mut channel_given_id = String::new();
                                let mut channel_given_token = String::new();

                                match channel_data.get("channel_id") {
                                    Some(value) => channel_given_id = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given id: {}", channel_given_id), LogLevel::DEBUG);

                                match channel_data.get("token") {
                                    Some(value) => channel_given_token = value.to_string(),
                                    None => (),
                                }

                                let req_user: Statement;
                                let mut user_exists = 0;

                                // SQL Request, remove users from channel
                                req_user = conn.prep("SELECT IF (COUNT(id) > 0, TRUE, FALSE) AS user_exists, id FROM user WHERE token = :u_token AND id != 0")?;

                                // Response
                                let res_user: Vec<mysql::Row> = conn.exec(
                                    &req_user,
                                    params! {
                                        "u_token" => channel_given_token.clone(),
                                    },
                                )?;

                                for mut row in res_user {
                                    // Getting user from db
                                    user_exists = row.take("user_exists").unwrap();
                                }

                                if user_exists == 1 {
                                    let req_delete_user_channel: Statement;

                                    // SQL Request, remove users from channel
                                    req_delete_user_channel = conn.prep("DELETE FROM user_channel WHERE id_channel = :c_id")?;

                                    // Response
                                    let _res_delete_user_channel: Vec<mysql::Row> = conn.exec(
                                        &req_delete_user_channel,
                                        params! {
                                            "c_id" => channel_given_id.clone(),
                                        },
                                    )?;

                                    let req_delete_channel: Statement;

                                    // SQL Request, delete channel
                                    req_delete_channel = conn.prep("DELETE FROM channel WHERE id = :c_id")?;

                                    // Response
                                    let _res_delete_channel: Vec<mysql::Row> = conn.exec(
                                        &req_delete_channel,
                                        params! {
                                            "c_id" => channel_given_id,
                                        },
                                    )?;

                                    return_data_json.insert("channel", String::from("OK"));
                                } else {
                                    return_data_json.insert("channel", String::from("KO"));
                                }
                            },
                            "set" => {
                                get_logger().log(format!("Action: Channel add"), LogLevel::TRACE);

                                let mut channel_given_token = String::new();
                                let mut channel_given_name = String::new();
                                let mut channel_given_description = String::new();

                                match channel_data.get("token") {
                                    Some(value) => channel_given_token = value.to_string(),
                                    None => (),
                                }

                                match channel_data.get("name") {
                                    Some(value) => channel_given_name = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given name: {}", channel_given_name), LogLevel::DEBUG);

                                match channel_data.get("description") {
                                    Some(value) => channel_given_description = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given description: {}", channel_given_description), LogLevel::DEBUG);

                                let req_select_user: Statement;
                                let mut res_select_user: Vec<mysql::Row> = Vec::new();

                                // SQL Request, check if token OK
                                req_select_user = conn.prep("SELECT IF (COUNT(id) > 0, TRUE, FALSE) AS user_exists, id FROM user WHERE token = :u_token AND id != 0")?;

                                // Response
                                res_select_user = conn.exec(
                                    &req_select_user,
                                    params! {
                                        "u_token" => channel_given_token.clone(),
                                    },
                                )?;

                                let mut user_exists = 0;

                                for mut row in res_select_user {
                                    // Getting user from db
                                    user_exists = row.take("user_exists").unwrap();
                                }

                                if user_exists == 1 {
                                    let req_insert_channel: Statement;
                                    let mut res_insert_channel: Vec<mysql::Row> = Vec::new();

                                    // SQL Request, insert channel
                                    req_insert_channel = conn.prep("INSERT INTO channel (name, description) VALUES (:c_name, :c_description)")?;

                                    // Response
                                    res_insert_channel = conn.exec(
                                        &req_insert_channel,
                                        params! {
                                            "c_name" => channel_given_name,
                                            "c_description" => channel_given_description,
                                        },
                                    )?;

                                    let req_insert_user_channel: Statement;
                                    let mut res_insert_user_channel: Vec<mysql::Row> = Vec::new();

                                    // SQL Request, insert user_channel
                                    req_insert_user_channel = conn.prep("INSERT INTO user_channel (id_user, id_channel) VALUES ((SELECT id FROM user WHERE token = :u_token), :c_id)")?;

                                    // Response
                                    res_insert_user_channel = conn.exec(
                                        &req_insert_user_channel,
                                        params! {
                                            "u_token" => channel_given_token,
                                            "c_id" => conn.last_insert_id(),
                                        },
                                    )?;

                                    return_data_json.insert("channel", String::from("OK"));
                                } else {
                                    return_data_json.insert("channel", String::from("KO"));
                                }
                            },
                            "add_user" => {
                                get_logger().log(format!("Action: Channel add user"), LogLevel::TRACE);

                                let mut channel_given_token = String::new();
                                let mut channel_given_login = String::new();
                                let mut channel_given_channel_id = String::new();

                                match channel_data.get("token") {
                                    Some(value) => channel_given_token = value.to_string(),
                                    None => (),
                                }

                                match channel_data.get("login") {
                                    Some(value) => channel_given_login = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given user login: {}", channel_given_login), LogLevel::DEBUG);

                                match channel_data.get("channel_id") {
                                    Some(value) => channel_given_channel_id = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given channel id: {}", channel_given_channel_id), LogLevel::DEBUG);

                                let req_select_post_user: Statement;
                                let mut res_select_post_user: Vec<mysql::Row> = Vec::new();

                                // SQL Request, check if token OK
                                req_select_post_user = conn.prep("SELECT IF (COUNT(id) > 0, TRUE, FALSE) AS user_exists FROM user WHERE token = :u_token AND id != 0")?;

                                // Response
                                res_select_post_user = conn.exec(
                                    &req_select_post_user,
                                    params! {
                                        "u_token" => channel_given_token.clone(),
                                    },
                                )?;

                                let mut user_exists = 0;

                                for mut row in res_select_post_user.clone() {
                                    // Getting user from db
                                    user_exists = row.take("user_exists").unwrap();
                                }

                                if user_exists == 1 {
                                    let req_select_user: Statement;
                                    let mut res_select_user: Vec<mysql::Row> = Vec::new();

                                    // SQL Request, check if token OK
                                    req_select_user = conn.prep("SELECT id FROM user WHERE username = :u_login OR email = :u_login")?;

                                    // Response
                                    res_select_user = conn.exec(
                                        &req_select_user,
                                        params! {
                                            "u_login" => channel_given_login.clone(),
                                        },
                                    )?;

                                    println!("{:#?}", res_select_user);

                                    if !res_select_user.is_empty() {
                                        let mut user_id = 0;

                                        for mut row in res_select_user {
                                            // Getting user from db
                                            user_id = row.take("id").unwrap();
                                        }

                                        let req_insert_user_channel: Statement;
                                        let mut res_insert_user_channel: Vec<mysql::Row> = Vec::new();

                                        // SQL Request, insert channel
                                        req_insert_user_channel = conn.prep("INSERT IGNORE INTO user_channel (id_user, id_channel) VALUES (:u_id, :c_id)")?;

                                        // Response
                                        res_insert_user_channel = conn.exec(
                                            &req_insert_user_channel,
                                            params! {
                                                "u_id" => user_id,
                                                "c_id" => channel_given_channel_id,
                                            },
                                        )?;

                                        return_data_json.insert("channel", String::from("OK"));
                                    } else {
                                        return_data_json.insert("channel", String::from("User to add does not exist!"));
                                    }
                                } else {
                                    return_data_json.insert("channel", String::from("Not a legitimate request..."));
                                }
                            },
                            "get_users" => {
                                get_logger().log(format!("Action: Channel get users"), LogLevel::TRACE);

                                let mut channel_given_token = String::new();
                                let mut channel_given_channel_id = String::new();

                                match channel_data.get("token") {
                                    Some(value) => channel_given_token = value.to_string(),
                                    None => (),
                                }

                                match channel_data.get("channel_id") {
                                    Some(value) => channel_given_channel_id = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given channel id: {}", channel_given_channel_id), LogLevel::DEBUG);

                                let req_select_post_user: Statement;
                                let mut res_select_post_user: Vec<mysql::Row> = Vec::new();

                                // SQL Request, check if token OK
                                req_select_post_user = conn.prep("SELECT IF (COUNT(id) > 0, TRUE, FALSE) AS user_exists FROM user WHERE token = :u_token AND id != 0")?;

                                // Response
                                res_select_post_user = conn.exec(
                                    &req_select_post_user,
                                    params! {
                                        "u_token" => channel_given_token.clone(),
                                    },
                                )?;

                                let mut user_exists = 0;

                                for mut row in res_select_post_user {
                                    // Getting user from db
                                    user_exists = row.take("user_exists").unwrap();
                                }

                                if user_exists == 1 {
                                    let req_select_user_channel: Statement;
                                    let mut res_select_user_channel: Vec<mysql::Row> = Vec::new();

                                    // SQL Request, insert channel
                                    req_select_user_channel = conn.prep("SELECT u.email, u.username FROM user_channel uc LEFT JOIN user u ON uc.id_user = u.id LEFT JOIN channel c ON uc.id_channel = c.id WHERE uc.id_channel = :c_id")?;

                                    // Response
                                    res_select_user_channel = conn.exec(
                                        &req_select_user_channel,
                                        params! {
                                            "c_id" => channel_given_channel_id,
                                        },
                                    )?;

                                    let mut users: Vec<_> = Vec::new();

                                    for mut row in res_select_user_channel {
                                        // Getting messages from db
                                        let user = User {
                                            email: row.take("email").unwrap(),
                                            username: row.take("username").unwrap(),
                                        };

                                        users.push(user);
                                    }

                                    let users_serialized = serde_json::to_string(&users).unwrap();

                                    return_data_json.insert("users", users_serialized);
                                    return_data_json.insert("channel", String::from("OK"));
                                } else {
                                    return_data_json.insert("channel", String::from("Not a legitimate request..."));
                                }
                            },
                            _ => get_logger().log("Channel action does not exist".to_string(), LogLevel::ERROR)
                        }

                    },
                    _ => ()
                }

                tx.send(return_data_json).unwrap();

                Ok(())
            });

            // Getting result from tread
            let received = rx.recv().unwrap();

            // Sending final result
            return warp::reply::json(&received);
        }); 
    
    // URI POST: /api/message
    // with json data : { "token":"u_token", "action":"get", "channel_id":"c_id", "count":"m_count"; "min_message_id":"m_id" }
    // with json data : { "token":"u_token", "action":"set", "id":"c_id", "content":"m_content", "date":"m_date" }
    // To get messages
    let message = warp::path!("message")
        .and(warp::post())
        .and(warp::body::json())
        .map( move |request_data: HashMap<String, String>| {
            let message_data = request_data.clone();
            let mut return_data_json: HashMap<_, String> = HashMap::new();

            // For sending result from thread
            let (tx, rx) = mpsc::channel();

            // For message
            let mut message_rng = rng_message_route.clone();
            let message_private_key = private_key_message_route.clone();

            // Thread
            let _thread = thread::spawn(move || -> Result<()> {
                let config = init::check_init_file();

                // Database connection
                let url: String = "mysql://".to_owned() + &*config.user + ":" + &*config.passwd + "@localhost:3306/" + &*config.database;
                let opts: Opts = Opts::from_url(&*url)?;
                let pool: Pool = Pool::new(opts)?;
                let mut conn: PooledConn = pool.get_conn()?;

                match message_data.get("action") {
                    Some(action) => {
                        match action.as_ref() {
                            "get" => {
                                get_logger().log(format!("Action: Message get"), LogLevel::TRACE);

                                let mut message_given_token = String::new();
                                let mut message_given_channel_id = String::new();
                                let mut message_given_count = String::new();
                                let mut message_given_message_id = String::new();

                                match message_data.get("token") {
                                    Some(value) => message_given_token = value.to_string(),
                                    None => (),
                                }

                                match message_data.get("channel_id") {
                                    Some(value) => message_given_channel_id = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given channel id: {}", message_given_channel_id), LogLevel::DEBUG);

                                match message_data.get("count") {
                                    Some(value) => message_given_count = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given count: {}", message_given_count), LogLevel::DEBUG);

                                match message_data.get("min_message_id") {
                                    Some(value) => message_given_message_id = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given message id: {}", message_given_message_id), LogLevel::DEBUG);

                                let req_select_message: Statement;
                                let mut res_select_message: Vec<mysql::Row> = Vec::new();

                                // SQL Request
                                req_select_message = conn.prep("SELECT * FROM message m LEFT JOIN user u ON m.id_user = u.id WHERE m.id_channel = :c_id AND m.id > :m_id ORDER BY m.id DESC LIMIT :m_count")?;

                                // Response
                                res_select_message = conn.exec(
                                    &req_select_message,
                                    params! {
                                        "c_id" => message_given_channel_id,
                                        "m_count" => message_given_count,
                                        "m_id" => message_given_message_id
                                    },
                                )?;

                                // SQL Request
                                let req_select_user_public_key: Statement;
                                let mut res_select_user_public_key: Vec<mysql::Row> = Vec::new();
                                req_select_user_public_key = conn.prep("SELECT public_key FROM user WHERE token = :u_token")?;

                                // Response
                                res_select_user_public_key = conn.exec(
                                    &req_select_user_public_key,
                                    params! {
                                        "u_token" => &message_given_token,
                                    },
                                )?;
                                
                                // Parsing result
                                let mut user_public_key = String::new();
                                for mut row in res_select_user_public_key {
                                    user_public_key = row.take("public_key").unwrap();
                                }

                                // Parsing response
                                let mut messages: Vec<_> = Vec::new();

                                for mut row in res_select_message {
                                    // Getting messages from db
                                    // Encrypting
                                    let content: String = row.take("content").unwrap();
                                    let new_content = encrypt::encrypt_message(&content, message_rng, RsaPublicKey::from_pkcs1_pem(&user_public_key).unwrap());
                                    
                                    let message = Message {
                                        id: row.take("id").unwrap(),
                                        content: new_content,
                                        date: row.take("date").unwrap(),
                                        username: row.take("username").unwrap(),
                                    };

                                    messages.push(message);
                                }

                                messages.reverse();
                                let messages_serialized = serde_json::to_string(&messages).unwrap();

                                get_logger().log(format!("Serialized messages sent"), LogLevel::DEBUG);
                 
                                return_data_json.insert("messages", messages_serialized);
                            },
                            "set" => {
                                get_logger().log(format!("Action: Message add"), LogLevel::TRACE);

                                let mut message_given_token = String::new();
                                let mut message_given_channel_id = String::new();
                                let mut message_given_content = String::new();
                                let mut message_given_date = String::new();

                                match message_data.get("token") {
                                    Some(value) => message_given_token = value.to_string(),
                                    None => (),
                                }

                                match message_data.get("id") {
                                    Some(value) => message_given_channel_id = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given channel id: {}", message_given_channel_id), LogLevel::DEBUG);
                                
                                match message_data.get("content") {
                                    Some(value) => message_given_content = value.to_string(),
                                    None => (),
                                }

                                match message_data.get("date") {
                                    Some(value) => message_given_date = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given message date: {}", message_given_date), LogLevel::DEBUG);
                                
                                let req_select_channel: Statement;
                                let mut res_select_channel: Vec<mysql::Row> = Vec::new();
                                let mut channel_exists = 0;

                                // SQL Request
                                req_select_channel = conn.prep("SELECT IF (COUNT(id) > 0, TRUE, FALSE) AS channel_exists FROM channel WHERE id = :c_id")?;

                                // Response
                                res_select_channel = conn.exec(
                                    &req_select_channel,
                                    params! {
                                        "c_id" => message_given_channel_id.clone(),
                                    },
                                )?;

                                for mut row in res_select_channel {
                                    channel_exists = row.take("channel_exists").unwrap();
                                }

                                if channel_exists != 0 {

                                    let req_select_message: Statement;
                                    let mut res_select_message: Vec<mysql::Row> = Vec::new();
                                    
                                    // Decrypting
                                    let message_encrypted: Vec<u8> = serde_json::from_str(&message_given_content).unwrap();
                                    let message_given_content_decrypted = encrypt::decrypt_message(message_encrypted, message_private_key);

                                    if message_given_content_decrypted.len() <= 250 {

                                        // SQL Request
                                        req_select_message = conn.prep("INSERT INTO message (content, date, id_user, id_channel) VALUES (:m_content, :m_date, (SELECT id FROM user WHERE token = :u_token), :c_id)")?;

                                        // Response
                                        res_select_message = conn.exec(
                                            &req_select_message,
                                            params! {
                                                "m_content" => message_given_content_decrypted,
                                                "m_date" => message_given_date,
                                                "u_token" => message_given_token,
                                                "c_id" => message_given_channel_id,
                                            },
                                        )?;

                                        return_data_json.insert("message", String::from("OK"));
                                        return_data_json.insert("channel", String::from("OK"));

                                    } else {
                                        
                                        return_data_json.insert("message", String::from("KO"));

                                    }
                                } else {
                                    return_data_json.insert("message", String::from("KO"));
                                    return_data_json.insert("channel", String::from("KO"));
                                }
                            },
                            _ => get_logger().log("Message action does not exist".to_string(), LogLevel::ERROR)
                        }
                    },
                    _ => get_logger().log("Channel action does not exist".to_string(), LogLevel::ERROR)
                }

                tx.send(return_data_json).unwrap();

                Ok(())
            });

            // Getting result from tread
            let received = rx.recv().unwrap();

            // Sending final result
            return warp::reply::json(&received);
        });

    // URI POST: /api/register
    // with json data : { "action":"set", "username":"user_pseudo", "email":"user_email", "password":"user_password" }
    // For registering user
    let user_register = warp::path!("register")
        .and(warp::post())
        .and(warp::body::json())
        .map( move |request_data: HashMap<String, String>| {
            let message_data = request_data.clone();
            let mut return_data_json: HashMap<_, String> = HashMap::new();

            // For sending result from thread
            let (tx, rx) = mpsc::channel();

            // Thread
            let _thread = thread::spawn(move || -> Result<()> {
                let config = init::check_init_file();

                // Database connection
                let url: String = "mysql://".to_owned() + &*config.user + ":" + &*config.passwd + "@localhost:3306/" + &*config.database;
                let opts: Opts = Opts::from_url(&*url)?;
                let pool: Pool = Pool::new(opts)?;
                let mut conn: PooledConn = pool.get_conn()?;

                match message_data.get("action") {
                    Some(action) => {
                        match action.as_ref() {
                            "set" => {
                                get_logger().log(format!("Action: Register user"), LogLevel::TRACE);

                                let mut message_given_username = String::new();
                                let mut message_given_email = String::new();
                                let mut message_given_password = String::new();

                                match message_data.get("username") {
                                    Some(value) => message_given_username = value.to_string(),
                                    None => (),
                                }

                                match message_data.get("email") {
                                    Some(value) => message_given_email = value.to_string(),
                                    None => (),
                                }
                                
                                match message_data.get("password") {
                                    Some(value) => message_given_password = value.to_string(),
                                    None => (),
                                }

                                // Check email and pseudo
                                
                                // SQL Request
                                let req_select_user = conn.prep(
                                    "SELECT * FROM `user` WHERE `email` = :email OR `username` = :username ",
                                )?;

                                // Response
                                let res_select_user: Vec<mysql::Row> = conn.exec(
                                    &req_select_user,
                                    params! {
                                        "email" => message_given_email.clone(),
                                        "username" => message_given_username.clone(),
                                    },
                                )?;

                                // Parsing response
                                let mut username_from_db = String::new();
                                let mut email_from_db = String::new();

                                for mut row in res_select_user {
                                    username_from_db = row.take("username").unwrap();
                                    email_from_db = row.take("email").unwrap();
                                }

                                if username_from_db == message_given_username || email_from_db == message_given_email {
                                    return_data_json.insert("registration", String::from("KO"));
                                    return_data_json.insert("description", String::from("Username or email already taken."));
                                } else{
                                    // Hashing password
                                    let hash_setup = "$6$salt";
                                    let hashed_password = sha512_crypt::hash_with(hash_setup, message_given_password).unwrap();
                                    
                                    let req_insert_user: Statement;

                                    // SQL Request
                                    req_insert_user = conn.prep("INSERT INTO user (username, email, password) VALUES (:username, :email, :password)")?;

                                    

                                    // Response
                                    let res_insert_user: Vec<mysql::Row> = conn.exec(
                                        &req_insert_user,
                                        params! {
                                            "username" => message_given_username,
                                            "email" => message_given_email,
                                            "password" => hashed_password
                                        },
                                    )?;

                                    return_data_json.insert("registration", String::from("OK"));
                                }
                            },
                            _ => get_logger().log("Register action does not exist".to_string(), LogLevel::ERROR)
                        }
                    },
                    _ => get_logger().log("Register action does not exist".to_string(), LogLevel::ERROR)
                }

                tx.send(return_data_json).unwrap();

                Ok(())
            });        

            // Getting result from tread
            let received = rx.recv().unwrap();

            // Sending final result
            return warp::reply::json(&received);
        });

    // Build routes
    let routes = user_login.or(user_register).or(channel).or(message);
    let routes = warp::path("api").and(routes);

    // Bind ip address and port
    warp::serve(routes.clone())
        .tls()
        .cert_path("certificates/ruslychat.afvr.pro.crt")
        .key_path("certificates/ruslychat.afvr.pro.key")
        .run(([0, 0, 0, 0], 6969))
        .await;
}

#[cfg(test)]
mod tests {
   
    use super::*;

    #[test]
    fn test_rsa_encryption_decryption() {

        // Conf 1
        let mut rng1 = OsRng;
        let private_key1 = RsaPrivateKey::new(&mut rng1, 1024).expect("failed to generate a key");

        // Conf 2
        let mut rng2 = OsRng;
        let private_key2 = RsaPrivateKey::new(&mut rng2, 1024).expect("failed to generate a key");
        let public_key2 = RsaPublicKey::from(&private_key2);

        let data_to_encrypt = "Hello";
        let data_encrypted = encrypt::encrypt_message(data_to_encrypt, rng1, public_key2);
        let data_decrypted = encrypt::decrypt_message(data_encrypted, private_key2);

        assert_eq!(data_to_encrypt.to_owned(), data_decrypted);
        // assert_ne!(data_to_encrypt.to_owned(), data_encrypted);
    }

    #[test]
    fn test_log_directory(){
        let config = init::check_init_file();
        assert_eq!(log::check_log_directory(config.logs_directory),true);
    }
}
