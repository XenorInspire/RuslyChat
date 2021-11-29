extern crate mysql;
extern crate chrono;

mod init;
mod log;
mod tcp_listening;

use mysql::prelude::*;
use mysql::*;
use log::LogLevel;
use log::Logger;

struct User {
    email: String,
    username: String,
    password: String,
}

fn main() -> Result<()> {
    let tcp_config = init::check_init_file();
    let config = init::check_init_file();
    tcp_listening::start_listening(tcp_config);

    let url: &str = "mysql://rust:rust@localhost:3306/ruslychat";
    let opts: Opts = Opts::from_url(url)?;

    let pool: Pool = Pool::new(opts)?;
    let mut conn: PooledConn = pool.get_conn()?;


    let users = vec![
        User { email: "coucou".to_string(), username: "toto".to_string(), password: "coucouToto".to_string() },
    ];

    conn.exec_batch(r"INSERT INTO user (email, username, password) VALUES (:email, :username, AES_ENCRYPT(:password, 'ruslychatAKATheBestChatEver'))",
                    users.iter().map(|p| params! {
                        "email" => p.email.clone(),
                        "username" => p.username.clone(),
                        "password" => p.password.clone(),
                    })
    )?;

    println!("{}", config.port);
    println!("{}", config.logs_directory);
    println!("{}", config.database);
    println!("{}", config.user);
    println!("{}", config.passwd);

    let mut logger = Logger {
        path: config.logs_directory,
        log_file: "".to_string(),
        max_size: 10
    };

    logger.log("coucou".to_string(), LogLevel::INFO);
    
    Ok(())
}