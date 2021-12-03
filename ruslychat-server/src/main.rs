extern crate chrono;
extern crate mysql;

mod api;
mod init;
mod log;
mod tcp_listening;

use log::LogLevel;
use log::Logger;
use mysql::prelude::*;
use mysql::*;
use pwhash::sha512_crypt;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::fs;
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
    // let thread1 = thread::spawn(|| {
    //     // hashed password test WIP
    //     let h = "$6$salt";
    //     let hash = sha512_crypt::hash_with(h, "tests").unwrap();
    //     if sha512_crypt::verify("tests", &hash) == true {
    //         println!("ok");
    //     } else {
    //         println!("ko");
    //     }
    // });

    // let thread2 = thread::spawn(move || -> Result<()> {
    //     let url: &str = "mysql://root:root@localhost:3306/rusly_db";
    //     let opts: Opts = Opts::from_url(url)?;

    //     let pool: Pool = Pool::new(opts)?;
    //     let mut conn: PooledConn = pool.get_conn()?;

    //     let users = vec![User {
    //         email: "ttt".to_string(),
    //         username: "ttt".to_string(),
    //         password: "coucouToto".to_string(),
    //     }];

    //     let stmt = conn.prep("SELECT * FROM `user` WHERE `id` = :id ")?;

    //     let res:Vec<Row> = conn.exec(&stmt,params! {
    //         "id" => 1,
    //     })?;
    //     // println!("{:?}",res);
    //     Ok(())
    // });

    // POST user login test WIP
    let user_login = warp::path!("login")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request_data: HashMap<String, String>| {
            let unhashed_pwd = request_data.clone();

            // Check password
            let thread = thread::spawn(move || -> Result<()> {
                let url: &str = "mysql://root:root@localhost:3306/rusly_db";
                let opts: Opts = Opts::from_url(url)?;

                let pool: Pool = Pool::new(opts)?;
                let mut conn: PooledConn = pool.get_conn()?;

                let stmt = conn.prep("SELECT * FROM `user` WHERE `id` = :id ")?;

                let res: Vec<mysql::Row> = conn.exec(
                    &stmt,
                    params! {
                        "id" => 1,
                    },
                )?;
                for row in res {
                    // Getting the hash value and compare with the hashed password given WIP

                    //Hasing the password
                    println!("{:?}", unhashed_pwd.get("pwd"));
                    let hash_from_db = row.unwrap().get(0).take();

                    let hash_setup = "$6$salt";
                    let hashed_pwd = sha512_crypt::hash_with(hash_setup, "unhashed_pwd").unwrap();

                    // Good password
                    // if hashed_pwd.to_string() == hash_from_db.unwrap() {
                    //     let test = HashMap::from([("Response", "Ok")]);
                    //     warp::reply::json(&test)
                    // } else {
                    //     let test = HashMap::from([("Response", "Ko")]);
                    //     warp::reply::json(&test)
                    // }
                    // println!("{:?}",row);
                    // println!("{:?}",row.unwrap().get(0).take());
                    // println!("{:?}",row.unwrap().get(0));
                }
                Ok(())
            });

            // let json = json::encode(&request_data).unwrap();
            let test = HashMap::from([("Response", "Ok")]);
            warp::reply::json(&test)
            // warp::reply::json(&request_data)
        });

    // GET user WIP
    let get_user = warp::path!("user" / u32).map(|id| format!("id {}", id));

    // Build routes
    let routes = user_login.or(get_user);
    let routes = warp::path("api").and(routes);

    warp::serve(routes).run(([127, 0, 0, 1], 6970)).await;
}

// TEST function
// pub async fn hello_fn() -> Result<impl warp::Reply> {
//     let our_ids = vec![1, 3, 7, 13];
//     warp::reply::json(&our_ids);
//     // if true {
//     //     Ok(println!("H"));
//     // } else {
//     //     Err(warp::reject::not_found());
//     // }
//     Ok(warp::reply::json(&our_ids))
// }

// fn main() -> Result<()> {

//     let tcp_config = init::check_init_file();
//     let config = init::check_init_file();
//     // tcp_listening::start_listening(tcp_config);

//     let url: &str = "mysql://root:root@localhost:3306/rusly_db";
//     let opts: Opts = Opts::from_url(url)?;

//     let pool: Pool = Pool::new(opts)?;
//     let mut conn: PooledConn = pool.get_conn()?;

//     let users = vec![
//         User { email: "c".to_string(), username: "h".to_string(), password: "coucouToto".to_string() },
//     ];

//     conn.exec_batch(r"INSERT INTO user (email, username, password) VALUES (:email, :username, :password)",
//                     users.iter().map(|p| params! {
//                         "email" => p.email.clone(),
//                         "username" => p.username.clone(),
//                         "password" => p.password.clone(),
//                     })
//     )?;

//     let mut logger = Logger {
//         path: config.logs_directory,
//         log_file: "".to_string(),
//         max_size: 10
//     };

//     logger.log("coucou".to_string(), LogLevel::INFO);
//     Ok(())
// }
