use crate::log;
use rsa::{PaddingScheme, PublicKey};

pub fn encrypt_message(
    message: &str,
    mut rng: rand::rngs::OsRng,
    pub_key: rsa::RsaPublicKey,
) -> std::vec::Vec<u8> {
    let message_encrypted = pub_key
        .encrypt(
            &mut rng,
            PaddingScheme::new_pkcs1v15_encrypt(),
            &message.as_bytes(),
        )
        .expect("failed to encrypt");

    log::get_logger().log("Message encrypted".to_string(), log::LogLevel::DEBUG);
    
    message_encrypted
}

pub fn decrypt_message(message: std::vec::Vec<u8>, priv_key: rsa::RsaPrivateKey) -> String {
    String::from_utf8(
        priv_key
            .decrypt(PaddingScheme::new_pkcs1v15_encrypt(), &message)
            .expect("failed to decrypt"),
    )
    .unwrap()
}
