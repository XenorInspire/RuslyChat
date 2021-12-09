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
            let unhashed_pwd = request_data.clone();

            let mut rs_data: HashMap<_, _> = HashMap::new();
            let mut rs_data2 = rs_data.clone();

            let (tx, rx) = mpsc::channel();

            // Check password
            let thread = thread::spawn(move || -> Result<()> {
                // Database connection
                let url: &str = "mysql://root:root@localhost:3306/rusly_db";
                let opts: Opts = Opts::from_url(url)?;
                let pool: Pool = Pool::new(opts)?;
                let mut conn: PooledConn = pool.get_conn()?;

                // SQL Request
                let stmt = conn.prep("SELECT * FROM `user` WHERE `id` = :id ")?;

                // Response
                let res: Vec<mysql::Row> = conn.exec(
                    &stmt,
                    params! {
                        "id" => 1,
                    },
                )?;

                // Parsing response
                for row in res {
                    // Getting the hash value and compare with the hashed password given WIP

                    // Getting hash from db
                    let password_from_db = unhashed_pwd.get("pwd");
                    let mut hash_from_db = String::new();

                    match password_from_db {
                        Some(value) => hash_from_db = value.to_string(),
                        None => (),
                    }

                    println!("{}", hash_from_db);

                    //Hashing the password
                    let given_password = request_data.get("pwd");
                    let mut user_password = String::new();
                    match given_password {
                        Some(value) => user_password = value.to_string(),
                        None => (),
                    }
                    println!("{}", user_password);

                    let hash_setup = "$6$salt";
                    let hashed_pwd = sha512_crypt::hash_with(hash_setup, user_password).unwrap();
                    println!("{}", hashed_pwd);

                    // Good password
                    if hashed_pwd == hash_from_db {
                        let test = HashMap::from([("Response", "Ok")]);
                        warp::reply::json(&test);
                    } else {
                        let test = HashMap::from([("Response", "Ko")]);
                        warp::reply::json(&test);
                    }
                    // println!("{:?}",row);
                    // println!("{:?}",row.unwrap().get(0).take());
                    // println!("{:?}",row.unwrap().get(0));
                }

                //rs_data = HashMap::from([("Response", "thread")]);
                rs_data.insert("Response", "thread");
                let val = String::from("hi");
                tx.send(rs_data).unwrap();
                // return warp::reply::json(&test);
                Ok(())
            });

            let received = rx.recv().unwrap();
            println!("Got: {:?}", received);

            // let mut rs_data2: HashMap<_, _>;
            // rs_data2.clear();
            // rs_data2.extend(rs_data.into_iter());

            // rs_data2.clone_from(&rs_data);

            // println!("{:?}", rs_data);
            // let rs_data2 = rs_data.clone();

            // rs_data2.extend(rs_data.iter().cloned());
            return warp::reply::json(&received);

            // let test1 = HashMap::from([(json_field, json_value)]);
            // if true {
            //     return warp::reply::json(&test1);
            // }

            // let json = json::encode(&request_data).unwrap();
            let test = HashMap::from([("Response", "OkGood")]);
            warp::reply::json(&test)

            // warp::reply::json(&request_data)
        });

    return user_login;
}
