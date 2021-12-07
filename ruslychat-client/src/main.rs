use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use std::io;
use std::str;

mod connect_tcp;
mod init;

fn main() {
    let config = init::check_init_file();

    let mut config = init::check_init_file();
    let mut backup = config.clone();
    let mut answer = String::from("1");

    while answer.eq("0") == false {
        println!("========================\n Welcome to RuslyChat !\n========================");
        println!("1 : Log in");
        println!("2 : Manage settings");
        println!("0 : Exit");

        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        match &*answer {
            "1" => {
                let mut rng = OsRng;
                let priv_key =
                    RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
                let pub_key = RsaPublicKey::from(&priv_key);
                connect_tcp::start_connection(config.clone(), priv_key, pub_key);
            }
            "2" => {
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
