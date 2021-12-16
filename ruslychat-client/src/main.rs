extern crate chrono;
extern crate ini;
extern crate rand;
extern crate rpassword;
extern crate rsa;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::env;
use std::io;

mod channel;
mod init;
mod log;
mod login;
mod message;
mod user;

fn main() {
    let mut config = init::check_init_file();
    let mut backup = config.clone();
    let mut answer = String::from("1");
    env::set_var("PATH_LOGGER", config.logs_directory.clone());
    log::get_logger().log("Ruslychat is starting".to_string(), log::LogLevel::INFO);

    let mut rng = OsRng;
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    log::get_logger().log("Ruslychat started!".to_string(), log::LogLevel::INFO);
    print!("\x1B[2J\x1B[1;1H");
    // Display main menu
    while answer.eq("0") == false {
        println!("========================\n Welcome to RuslyChat !\n========================");
        println!("1 : Log in");
        println!("2 : Register to RuslyChat");
        println!("3 : Manage settings");
        println!("0 : Exit");

        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        match &*answer {
            // Log in process
            "1" => {
                if login::request_login(config.clone(), pub_key.clone()) == 0 {
                    channel::display_main_menu(
                        config.domain.clone(),
                        config.port_dest.clone().to_string(),
                        priv_key.clone(),
                        rng.clone(),
                    )
                }
            }
            // Registration process
            "2" => {
                login::user_registration(
                    config.domain.clone(),
                    config.port_dest.clone().to_string(),
                );
            }

            // Settings management process
            "3" => {
                config = init::change_config_values(config);
                if config != backup {
                    init::create_new_config_file(init::CURRENT_CONFIG_FILE_MODE, config.clone());
                    backup = config.clone();
                }
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsa_encryption_decryption() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 1024).expect("failed to generate a key");
        let public_key = RsaPublicKey::from(&private_key);

        let data_to_encrypt = "Hello";
        let data_encrypted = message::encrypt_message(data_to_encrypt, rng, public_key);
        let data_decrypted = message::decrypt_message(data_encrypted, private_key);

        assert_eq!(data_to_encrypt.to_owned(), data_decrypted);
    }

    #[test]
    fn test_connection_api() {
        let config = init::check_init_file();
        let mut rng = OsRng;
        let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);

        assert_eq!(
            login::api_login(
                config.domain.clone(),
                config.port_dest.clone().to_string(),
                "test".to_string(),
                "test".to_string(),
                pub_key.clone(),
            ),
            0
        );
    }

    #[test]
    fn test_config_file() {
        let config = init::check_init_file();
        init::create_new_config_file(init::NEW_CONFIG_FILE_MODE, config);
        let new_config = init::check_init_file();

        assert_eq!(new_config.domain.is_empty(), false);
        assert_eq!(new_config.logs_directory.is_empty(), false);
        assert_eq!(new_config.port_dest.to_string().is_empty(), false);
    }
}
