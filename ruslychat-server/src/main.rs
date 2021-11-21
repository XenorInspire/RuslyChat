extern crate mysql;

mod init;

use mysql::prelude::*;
use mysql::*;

struct User {
    email: String,
    username: String,
    password: String,
}

fn main() -> Result<()> {
    let config = init::check_init_file();

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

    Ok(())
}