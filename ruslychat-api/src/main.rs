extern crate chrono;
extern crate mysql;
extern crate pwhash;
extern crate rand;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate warp;
extern crate ini;

// mod api;
mod init;
mod log;
mod encrypt;

use ini::Ini;
use rand::rngs::OsRng;
use rsa::{pkcs1::FromRsaPublicKey, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey, pkcs1::ToRsaPublicKey, pkcs1::ToRsaPrivateKey, pkcs1::FromRsaPrivateKey};
use log::{get_logger, LogLevel};
use mysql::prelude::*;
use mysql::*;
use pwhash::sha512_crypt;
use serde::Serialize;
use std::collections::HashMap;
//use std::convert::Infallible;
use std::sync::mpsc;
use std::thread;
//use std::time::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::env;
//use warp::http::StatusCode;
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
    content: String,
    date: String,
}

#[tokio::main]
async fn main() {
    let config = init::check_init_file();
    env::set_var("PATH_LOGGER_API", config.logs_directory.clone());
    get_logger().log("Ruslychat API is starting...".to_string(), LogLevel::INFO);

    let mut rng = OsRng;
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);
    
    let public_key_string = format!("{:?}", ToRsaPublicKey::to_pkcs1_pem(&pub_key).unwrap());
    let private_key_string = format!("{:?}", ToRsaPrivateKey::to_pkcs1_pem(&priv_key).unwrap());
    
    let mut conf = Ini::new();
    conf.with_section(Some("KEYS"))
        .set("private_key", private_key_string)
        .set("public_key", public_key_string);
    conf.write_to_file("keys.ini").unwrap();

    /*let mut rng = OsRng;
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);*/
    get_logger().log("Ruslychat API started!".to_string(), LogLevel::INFO);
    
    // URI POST: /api/login
    // with json data : { "login":"pseudo", "password":"password" }
    // For first login and generating the token
    let user_login = warp::path!("login")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request_data: HashMap<String, String>| {
            let user_data = request_data.clone();
            let mut return_data_json: HashMap<_, String> = HashMap::new();

            // For sending result from thread
            let (tx, rx) = mpsc::channel();

            // Checking password in thread
            let _thread = thread::spawn(move || -> Result<()> {
                let config = init::check_init_file();

                let mut user_given_id = String::new();
                match user_data.get("login") {
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

                get_logger().log(
                    format!("res_select_user: {:?}", res_select_user),
                    LogLevel::DEBUG,
                );

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

                    get_logger().log(format!("token: {}", token), LogLevel::DEBUG);

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

                    return_data_json.insert("connection", "Success".to_string());
                    return_data_json.insert("token", token);
                    tx.send(return_data_json).unwrap();
                } else {
                    return_data_json.insert("connection", "Refused".to_string());
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
                                get_logger().log(format!("Given token: {}", channel_given_token), LogLevel::DEBUG);

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

                                get_logger().log(format!("res_select_channel: {:?}", res_select_channel), LogLevel::DEBUG);

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

                                get_logger().log(format!("Serialized channels: {}", channels_serialized), LogLevel::DEBUG);

                                return_data_json.insert("channels", channels_serialized);
                            },
                            "del" => {
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
                                get_logger().log(format!("Given token: {}", channel_given_token), LogLevel::DEBUG);

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
                            },
                            "set" => {
                                let mut channel_given_token = String::new();
                                let mut channel_given_name = String::new();
                                let mut channel_given_description = String::new();

                                match channel_data.get("token") {
                                    Some(value) => channel_given_token = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given token: {}", channel_given_token), LogLevel::DEBUG);

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
                                req_select_user = conn.prep("SELECT IF (COUNT(id) > 0, TRUE, FALSE) AS user_exists, id FROM user WHERE token = :u_token")?;

                                // Response
                                res_select_user = conn.exec(
                                    &req_select_user,
                                    params! {
                                        "u_token" => channel_given_token.clone(),
                                    },
                                )?;

                                get_logger().log(format!("res_select_user: {:?}", res_select_user), LogLevel::DEBUG);

                                let mut user_exists = 0;
                                let mut user_id = 0;

                                for mut row in res_select_user {
                                    // Getting user from db
                                    user_exists = row.take("user_exists").unwrap();
                                    user_id = row.take("id").unwrap();
                                }

                                if user_exists == 1 && user_id != 0 {
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

                                    get_logger().log(format!("res_insert_channel: {:?}", res_insert_channel), LogLevel::DEBUG);

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

                                    get_logger().log(format!("res_insert_user_channel: {:?}", res_insert_user_channel), LogLevel::DEBUG);

                                    return_data_json.insert("channel", String::from("OK"));
                                } else {
                                    return_data_json.insert("channel", String::from("KO"));
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
        .map(|request_data: HashMap<String, String>| {
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
                            "get" => {
                                let mut message_given_token = String::new();
                                let mut message_given_channel_id = String::new();
                                let mut message_given_count = String::new();
                                let mut message_given_message_id = String::new();

                                match message_data.get("token") {
                                    Some(value) => message_given_token = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given token: {}", message_given_token), LogLevel::DEBUG);

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
                                req_select_message = conn.prep("SELECT * FROM message m LEFT JOIN user u ON m.id_user = u.id WHERE u.token = :u_token AND m.id_channel = :c_id AND m.id > :m_id ORDER BY m.id DESC LIMIT :m_count")?;

                                // Response
                                res_select_message = conn.exec(
                                    &req_select_message,
                                    params! {
                                        "u_token" => message_given_token,
                                        "c_id" => message_given_channel_id,
                                        "m_count" => message_given_count,
                                        "m_id" => message_given_message_id
                                    },
                                )?;

                                get_logger().log(format!("res_select_message: {:?}", res_select_message), LogLevel::DEBUG);

                                // Parsing response
                                let mut messages: Vec<_> = Vec::new();

                                for mut row in res_select_message {
                                    // Getting messages from db
                                    let message = Message {
                                        id: row.take("id").unwrap(),
                                        content: row.take("content").unwrap(),
                                        date: row.take("date").unwrap()
                                    };

                                    messages.push(message);
                                }

                                messages.reverse();
                                let messages_serialized = serde_json::to_string(&messages).unwrap();

                                get_logger().log(format!("Serialized messages: {}", messages_serialized), LogLevel::DEBUG);

                                return_data_json.insert("messages", messages_serialized);
                            },
                            "set" => {
                                let mut message_given_token = String::new();
                                let mut message_given_channel_id = String::new();
                                let mut message_given_content = String::new();
                                let mut message_given_date = String::new();

                                match message_data.get("token") {
                                    Some(value) => message_given_token = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given token: {}", message_given_token), LogLevel::DEBUG);

                                match message_data.get("id") {
                                    Some(value) => message_given_channel_id = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given channel id: {}", message_given_channel_id), LogLevel::DEBUG);
                                
                                match message_data.get("content") {
                                    Some(value) => message_given_content = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given message content: {}", message_given_content), LogLevel::DEBUG);

                                match message_data.get("date") {
                                    Some(value) => message_given_date = value.to_string(),
                                    None => (),
                                }
                                get_logger().log(format!("Given message date: {}", message_given_date), LogLevel::DEBUG);
                                
                                let req_select_message: Statement;
                                let mut res_select_message: Vec<mysql::Row> = Vec::new();

                                let conf = Ini::load_from_file("keys.ini").unwrap();
                                let keys_section = conf.section(Some("KEYS")).unwrap();
                                let private_key = keys_section.get("private_key").unwrap();
                                let public_key = keys_section.get("public_key").unwrap();

                                //println!("{}", &private_key[10..private_key.len() - 1]);
                                //println!("{}", public_key);
                                println!("{}", private_key);
                                println!("{}", public_key);

                                let message_encrypted: Vec<u8> = serde_json::from_str(&message_given_content).unwrap();
                                message_given_content = encrypt::decrypt_message(message_encrypted, RsaPrivateKey::from_pkcs1_pem(private_key).unwrap());
                                // message_given_content = "coucou c mwa".to_string();

                                // SQL Request
                                req_select_message = conn.prep("INSERT INTO message (content, date, id_user, id_channel) VALUES (:m_content, :m_date, (SELECT id FROM user WHERE token = :u_token), :c_id)")?;

                                // Response
                                res_select_message = conn.exec(
                                    &req_select_message,
                                    params! {
                                        "m_content" => message_given_content,
                                        "m_date" => message_given_date,
                                        "u_token" => message_given_token,
                                        "c_id" => message_given_channel_id,
                                    },
                                )?;

                                return_data_json.insert("message", String::from("OK"));
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

    // GET user data WIP
    let get_user = warp::path!("user" / u32).map(|id| format!("id {}", id));

    // Build routes
    let routes = user_login.or(get_user).or(channel).or(message);
    let routes = warp::path("api").and(routes);

    // Bind ip address and port
    warp::serve(routes).run(([0, 0, 0, 0], 6969)).await;
}
