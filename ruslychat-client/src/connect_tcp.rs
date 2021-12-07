use crate::init;
use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey, pkcs1::ToRsaPublicKey};
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const MSG_SIZE: usize = 500;

pub fn start_connection(
    config: init::Config,
    priv_key: rsa::RsaPrivateKey,
    pub_key: rsa::RsaPublicKey,
) {
    let mut client = TcpStream::connect(format!(
        "{}:{}",
        config.domain,
        config.port_dest.to_string()
    ))
    .expect("Connection failed, check your internet connection");
    client
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();
    let mut is_key_sent: bool = false;

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("message recv {:?}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection lost");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client
                    .write_all(&buff)
                    .expect("Error, your message can't be sent. Socket Failed.");
                println!("message sent {:?}", msg);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message :");
    loop {
        let msg;
        if is_key_sent == false {
            msg = format!("pk={:?}", ToRsaPublicKey::to_pkcs1_pem(&pub_key));
            println!("{}", msg);
            is_key_sent = true;
        } else {
            let mut buff = String::new();
            io::stdin()
                .read_line(&mut buff)
                .expect("Reading from stdin failed");
            msg = buff.trim().to_string();
        }
        if msg == "!quit" || tx.send(msg).is_err() {
            break;
        }
    }
    println!("bye bye!");
}

pub fn encrypt_message(
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

pub fn decrypt_message(message: std::vec::Vec<u8>, priv_key: rsa::RsaPrivateKey) -> String {
    String::from_utf8(
        priv_key
            .decrypt(PaddingScheme::new_pkcs1v15_encrypt(), &message)
            .expect("failed to decrypt"),
    )
    .unwrap()
}
