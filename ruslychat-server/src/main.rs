extern crate mysql;
extern crate chrono;

mod init;
mod log;
mod tcp_listening;

use mysql::prelude::*;
use mysql::*;
use log::LogLevel;
use log::Logger;
use std::str;
use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

struct User {
    email: String,
    username: String,
    password: String,
}

fn main() -> Result<()> {

    // Encryption test WIP

    // Encryption conf
    let mut rng = OsRng;
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    // Data to encrypt
    let data = "Message à chiffrer".as_bytes();
    // println!("{:?}",&data);

    // Encrypt data
    let enc_data = pub_key
    .encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), &data)
    .expect("failed to encrypt");
    // println!("{:?}",&enc_data);
    // println!("{:?}",&enc_data.iter().map(|&c| c as char).collect::<String>());
    
    // Decrypt data 
    let dec_data = priv_key
    .decrypt(PaddingScheme::new_pkcs1v15_encrypt(), &enc_data)
    .expect("failed to decrypt");
    // assert_eq!(&data[..], &dec_data[..]);
    println!("{:?}",str::from_utf8(&dec_data).unwrap());
    
    // let x = String::from_utf8(
    //     "The compiler é said “you have an error!”."
    //         .bytes()
    //         .flat_map(|b| std::ascii::escape_default(b))
    //         .collect::<Vec<u8>>(),
    // )
    // .unwrap();
    // println!("{}", x);

    // let text = b"hello";
    // let s = String::from_utf8_lossy(text);
    // println!("{}", s); // he�lo

    // some bytes, in a stack-allocated array
    // let sparkle_heart = [240, 159, 146, 150];
    // let sparkle_heart = b"Hello";

    // let sparkle_heart = str::from_utf8(&sparkle_heart).unwrap();
    // println!("{}",sparkle_heart);

    // let string = "Bojour é";
    // println!("{:?}", string.as_bytes());

    let tcp_config = init::check_init_file();
    let config = init::check_init_file();
    tcp_listening::start_listening(tcp_config);

    // let url: &str = "mysql://rust:rust@localhost:3306/ruslychat";
    // let opts: Opts = Opts::from_url(url)?;

    // let pool: Pool = Pool::new(opts)?;
    // let mut conn: PooledConn = pool.get_conn()?;


    // let users = vec![
    //     User { email: "coucou".to_string(), username: "toto".to_string(), password: "coucouToto".to_string() },
    // ];

    // conn.exec_batch(r"INSERT INTO user (email, username, password) VALUES (:email, :username, AES_ENCRYPT(:password, 'ruslychatAKATheBestChatEver'))",
    //                 users.iter().map(|p| params! {
    //                     "email" => p.email.clone(),
    //                     "username" => p.username.clone(),
    //                     "password" => p.password.clone(),
    //                 })
    // )?;

    let mut logger = Logger {
        path: config.logs_directory,
        log_file: "".to_string(),
        max_size: 10
    };

    logger.log("coucou".to_string(), LogLevel::INFO);
    
    Ok(())
}