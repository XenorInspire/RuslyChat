extern crate chrono;
extern crate mysql;

// mod api;
mod init;
mod log;

use log::LogLevel;
use log::Logger;
use mysql::prelude::*;
use mysql::*;
use pwhash::sha512_crypt;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::fs;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use warp::http::StatusCode;
use warp::Filter;

struct User {
    email: String,
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    // URI POST: /api/login
    // with json data : { "login":"pseudo", "pwd":"password" }
    // For first login and generating the token
    let user_login = warp::path!("login")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request_data: HashMap<String, String>| {
            let user_data = request_data.clone();
            let mut return_data_json: HashMap<_, _> = HashMap::new();

            // For sending result from thread
            let (tx, rx) = mpsc::channel();

            // Checking password in thread
            let thread = thread::spawn(move || -> Result<()> {
                let mut user_given_email = String::new();
                match user_data.get("login") {
                    Some(value) => user_given_email = value.to_string(),
                    None => (),
                }

                //DEBUG
                println!("given mail : {}", user_given_email);

                // Database connection
                let url: &str = "mysql://root:root@localhost:3306/rusly_db";
                let opts: Opts = Opts::from_url(url)?;
                let pool: Pool = Pool::new(opts)?;
                let mut conn: PooledConn = pool.get_conn()?;

                // SQL Request
                let stmt = conn.prep("SELECT * FROM `user` WHERE `email` = :email ")?;

                // Response
                let res: Vec<mysql::Row> = conn.exec(
                    &stmt,
                    params! {
                        "email" => user_given_email,
                    },
                )?;

                //DEBUG
                println!("res : {:?}", res);

                // Parsing response
                let mut hash_from_db = String::new();

                //Hashing the password given
                let given_password = request_data.get("pwd");
                let mut user_password = String::new();
                match given_password {
                    Some(value) => user_password = value.to_string(),
                    None => (),
                }
                println!("user given pwd : {}", user_password);

                let hash_setup = "$6$salt";
                let hashed_given_pwd = sha512_crypt::hash_with(hash_setup, user_password).unwrap();
                println!("hashed user given pwd : {}", hashed_given_pwd);

                for mut row in res {
                    //DEBUG
                    println!("First value of res : {:?}", row);
                    // Getting hashed from db
                    hash_from_db = row.take("password").unwrap();
                }
                // Good password
                if hashed_given_pwd == hash_from_db {
                    // Generating the token WIP
                    // ..........................//

                    return_data_json.insert("Connection", "Success");
                    tx.send(return_data_json).unwrap();
                } else {
                    return_data_json.insert("Connection", "Refused");
                    tx.send(return_data_json).unwrap();
                }
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
    let routes = user_login.or(get_user);
    let routes = warp::path("api").and(routes);

    // Bind ip address and port
    warp::serve(routes).run(([0, 0, 0, 0], 6970)).await;
}
