extern crate chrono;
extern crate mysql;

mod init;
mod log;
mod tcp_listening;

use log::LogLevel;
use log::Logger;
use mysql::prelude::*;
use mysql::*;
use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

fn main() {
    let config = init::check_init_file();
    println!("The server is starting...");
    
    // Encryption POC
    let mut rng = OsRng;
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    // Message to encrypt
    //let message = "Message à chiffrer";

    // Encryption
    //let encrypted_message = tcp_listening::encrypt_message(message, rng, pub_key);

    // Decryption
    //let decrypted_message = tcp_listening::decrypt_message(encrypted_message, priv_key);
    //println!("{:?}", &decrypted_message);

    tcp_listening::start_listening(config.clone(), priv_key, pub_key);

    let mut logger = Logger {
        path: config.logs_directory,
        log_file: "".to_string(),
        max_size: 10,
    };

    //logger.log("coucou".to_string(), LogLevel::INFO);
}
