use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey, pkcs1::FromRsaPublicKey};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::str;
use std::sync::mpsc;
use std::thread;

use crate::init;

const MSG_SIZE: usize = 500;

pub fn start_listening(
    config: init::Config,
    priv_key: rsa::RsaPrivateKey,
    pub_key: rsa::RsaPublicKey,
) {
    let server = TcpListener::bind(format!("0.0.0.0:{}", config.port.to_string()))
        .expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    println!("Server listening on port {}", config.port.to_string());

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));
            let mut is_key_sent: bool = false;

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        println!("{}", is_key_sent);

                        // Check if this is the public key of the client
                        if is_key_sent == false && msg.contains("-----BEGIN RSA PUBLIC KEY-----\n")
                            && msg.contains("\n-----END RSA PUBLIC KEY-----\n")
                        {
                            is_key_sent = true;
                            let client_pub_key = RsaPublicKey::from_pkcs1_pem(&*msg).unwrap();
                        }

                        println!("{}: {:?}", addr, msg);
                        tx.send("Message recu --> ".to_string() + &msg);
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }

                thread::sleep(::std::time::Duration::from_millis(100));
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }

        thread::sleep(::std::time::Duration::from_millis(100));
    }
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
