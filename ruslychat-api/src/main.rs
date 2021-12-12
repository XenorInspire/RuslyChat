extern crate chrono;
extern crate mysql;
extern crate rand;
extern crate pwhash;
extern crate warp;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

// mod api;
mod init;
mod log;

use log::LogLevel;
use log::Logger;
use mysql::prelude::*;
use mysql::*;
use pwhash::sha512_crypt;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::fs;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::env;
use rand::Rng;
use rand::distributions::Alphanumeric;
use warp::http::StatusCode;
use warp::Filter;
use std::iter::FromIterator;

#[derive(Serialize, Debug)]
struct Channel {
    id: u32,
    name: String,
    description: String,
}

#[derive(Serialize, Debug)]
struct Message {
    content: String,
    date: String,
}

#[tokio::main]
async fn main() {
    {
        let config = init::check_init_file();

        let mut logger = Logger {
            path: config.logs_directory,
            log_file: "".to_string(),
            max_size: 10
        };

        logger.log("API starting...".to_string(), LogLevel::INFO);
    }
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
            let thread = thread::spawn(move || -> Result<()> {
                let config = init::check_init_file();

                let mut logger = Logger {
                    path: config.logs_directory,
                    log_file: "".to_string(),
                    max_size: 10
                };

                let mut user_given_id = String::new();
                match user_data.get("login") {
                    Some(value) => user_given_id = value.to_string(),
                    None => (),
                }

                //DEBUG
                logger.log(format!("given login: {}", user_given_id), LogLevel::DEBUG);
                //println!("given login: {}", user_given_id);


                // Database connection
                let url: String = "mysql://".to_owned() + &*config.user + ":" + &*config.passwd + "@localhost:3306/" + &*config.database;
                let opts: Opts = Opts::from_url(&*url)?;
                let pool: Pool = Pool::new(opts)?;
                let mut conn: PooledConn = pool.get_conn()?;

                // SQL Request
                let req_select_user = conn.prep("SELECT * FROM `user` WHERE `email` = :email OR `username` = :username ")?;

                // Response
                let res_select_user: Vec<mysql::Row> = conn.exec(
                    &req_select_user,
                    params! {
                        "email" => user_given_id.clone(),
                        "username" => user_given_id,
                    },
                )?;

                //DEBUG
                logger.log(format!("res_select_user: {:?}", res_select_user), LogLevel::DEBUG);

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
                println!("hashed user given password: {}", hashed_given_pwd);

                for mut row in res_select_user {
                    //DEBUG
                    println!("First value of res_select_user: {:?}", row);
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

                    println!("token: {}", token);

                    // SQL Request
                    let req_update_user_token = conn.prep("UPDATE `user` SET `token` = :token WHERE `id` = :id")?;

                    // Response
                    let res_update_user_token: Vec<mysql::Row> = conn.exec(
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
    // with json data : { "token":"u_token", "action":"set", "id":"c_id", "name":"c_name", "description":"c_description" }
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
            let thread = thread::spawn(move || -> Result<()> {
                let config = init::check_init_file();

                let mut logger = Logger {
                    path: config.logs_directory,
                    log_file: "".to_string(),
                    max_size: 10
                };

                let mut channel_given_id = String::new();
                match channel_data.get("id") {
                    Some(value) => channel_given_id = value.to_string(),
                    None => (),
                }

                //DEBUG
                logger.log(format!("Given id: {}", channel_given_id), LogLevel::DEBUG);

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
                                println!("Given id: {}", channel_given_id);

                                match channel_data.get("token") {
                                    Some(value) => channel_given_token = value.to_string(),
                                    None => (),
                                }
                                println!("Given token: {}", channel_given_token);

                                let mut req_select_channel: Statement;
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

                                //DEBUG
                                println!("res_select_channel: {:?}", res_select_channel);

                                // Parsing response
                                let mut channels: Vec<_> = Vec::new();

                                for mut row in res_select_channel {
                                    // Getting channel from db
                                    println!("value of res_select_channel: {:?}", row);

                                    let channel = Channel {
                                        id: row.take("id").unwrap(),
                                        name: row.take("name").unwrap(),
                                        description: row.take("description").unwrap()
                                    };

                                    channels.push(channel);
                                }

                                let channels_serialized = serde_json::to_string(&channels).unwrap();
                                println!("Serialized channels: {}", channels_serialized);
                                println!("{:#?}", channels);

                                return_data_json.insert("channels", channels_serialized);
                            },
                            "del" => {

                            },
                            "set" => {

                            },
                            _ => logger.log("Channel action does not exist".to_string(), LogLevel::ERROR)
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
    // with json data : { "token":"u_token", "action":"get", "id":"c_id", "count":"m_count" }
    // with json data : { "token":"u_token", "action":"set", "count":"m_count" }
    // To get channels
    let message = warp::path!("message")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request_data: HashMap<String, String>| {
            let message_data = request_data.clone();
            let mut return_data_json: HashMap<_, String> = HashMap::new();

            // For sending result from thread
            let (tx, rx) = mpsc::channel();

            // Thread
            let thread = thread::spawn(move || -> Result<()> {
                let config = init::check_init_file();

                let mut logger = Logger {
                    path: config.logs_directory,
                    log_file: "".to_string(),
                    max_size: 10
                };

                let mut message_given_count = String::new();
                match message_data.get("count") {
                    Some(value) => message_given_count = value.to_string(),
                    None => (),
                }

                //DEBUG
                logger.log(format!("Given count: {}", message_given_count), LogLevel::DEBUG);

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

                                match message_data.get("token") {
                                    Some(value) => message_given_token = value.to_string(),
                                    None => (),
                                }
                                println!("Given token: {}", message_given_token);

                                match message_data.get("id") {
                                    Some(value) => message_given_channel_id = value.to_string(),
                                    None => (),
                                }
                                println!("Given channel id: {}", message_given_channel_id);

                                match message_data.get("count") {
                                    Some(value) => message_given_count = value.to_string(),
                                    None => (),
                                }
                                println!("Given count: {}", message_given_count);

                                let mut req_select_message: Statement;
                                let mut res_select_message: Vec<mysql::Row> = Vec::new();

                                // SQL Request
                                req_select_message = conn.prep("SELECT * FROM message m LEFT JOIN user u ON m.id_user = u.id WHERE u.token = :u_token AND m.id_channel = :c_id ORDER BY m.id DESC LIMIT :count")?;

                                // Response
                                res_select_message = conn.exec(
                                    &req_select_message,
                                    params! {
                                        "u_token" => message_given_token,
                                        "c_id" => message_given_channel_id,
                                        "count" => message_given_count,
                                    },
                                )?;

                                //DEBUG
                                println!("res_select_message: {:?}", res_select_message);

                                // Parsing response
                                let mut messages: Vec<_> = Vec::new();

                                for mut row in res_select_message {
                                    // Getting channel from db
                                    println!("value of res_select_message: {:?}", row);

                                    let message = Message {
                                        content: row.take("content").unwrap(),
                                        date: row.take("date").unwrap()
                                    };

                                    messages.push(message);
                                }

                                messages.reverse();
                                let messages_serialized = serde_json::to_string(&messages).unwrap();
                                println!("Serialized messages: {}", messages_serialized);
                                println!("{:#?}", messages);

                                return_data_json.insert("messages", messages_serialized);
                            },
                            "set" => {

                            },
                            _ => logger.log("Message action does not exist".to_string(), LogLevel::ERROR)
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

    // GET user data WIP
    let get_user = warp::path!("user" / u32).map(|id| format!("id {}", id));

    // Build routes
    let routes = user_login.or(get_user).or(channel).or(message);
    let routes = warp::path("api").and(routes);

    // Bind ip address and port
    warp::serve(routes).run(([0, 0, 0, 0], 6970)).await;
}
