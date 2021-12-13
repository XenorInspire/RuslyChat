use std::io;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Message {
    pub content: String,
    pub date: String,
}

pub fn chat(api_host: String, api_port: String) {

    let mut answer: String = String::new();

    while answer.ne("!exit") {
        let mut buff_chat = String::new();

        io::stdin()
            .read_line(&mut buff_chat)
            .expect("Reading from stdin failed");
        answer = buff_chat.trim().to_string();
    }

}

pub fn send_message(content: String, api_host: String, api_port: String) {
    
}

pub fn receive_message() {

}

fn encrypt_message(
    message: &str,
    mut rng: rand::rngs::OsRng,
    pub_key: rsa::RsaPublicKey,
) -> std::vec::Vec<u8> {
    pub_key
        .encrypt(
            &mut rng,
            PaddingScheme::new_pkcs1v15_encrypt(),
            &message.as_bytes(),
        )
        .expect("failed to encrypt")
}

fn decrypt_message(message: std::vec::Vec<u8>, priv_key: rsa::RsaPrivateKey) -> String {
    String::from_utf8(
        priv_key
            .decrypt(PaddingScheme::new_pkcs1v15_encrypt(), &message)
            .expect("failed to decrypt"),
    )
    .unwrap()
}

fn check_command(command: String) {

    

}