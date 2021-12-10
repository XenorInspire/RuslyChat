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

pub fn user_login() -> warp::filter::map::Map {
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

                let mut user_given_login = String::new();
                match user_data.get("login") {
                    Some(value) => user_given_login = value.to_string(),
                    None => (),
                }

                //DEBUG
                logger.log(format!("given login: {}", user_given_login), LogLevel::DEBUG);
                //println!("given login: {}", user_given_login);


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
                        "email" => user_given_login.clone(),
                        "username" => user_given_login,
                    },
                )?;

                //DEBUG
                println!("res_select_user: {:?}", res_select_user);

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
                println!("user given password: {}", user_password);

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
                    // Generating the token WIP
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

    user_login
}
