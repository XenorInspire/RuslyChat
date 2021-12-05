use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

extern crate openssl;

use openssl::rsa::{Padding, Rsa};
use openssl::symm::Cipher;

use crate::init;

const MSG_SIZE: usize = 500;

pub fn start_listening (config: init::Config) {

    let passphrase = "server_rust_by_example";

    let rsa = Rsa::generate(1024).unwrap();
    let private_key: Vec<u8> = rsa
        .private_key_to_pem_passphrase(Cipher::aes_128_cbc(), passphrase.as_bytes())
        .unwrap();
    let public_key: Vec<u8> = rsa.public_key_to_pem().unwrap();

    let rsa = Rsa::public_key_from_pem(&public_key).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    let _ = rsa
        .public_encrypt(passphrase.as_bytes(), &mut buf, Padding::PKCS1)
        .unwrap();

    let data = buf;

    // Decrypt with private key
    let rsa = Rsa::private_key_from_pem_passphrase(&private_key, passphrase.as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    let _ = rsa
        .private_decrypt(&data, &mut buf, Padding::PKCS1)
        .unwrap();

    let server = TcpListener::bind(format!(
        "0.0.0.0:{}",
        config.port.to_string()
    )).expect("Listener failed to bind");
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    println!("Server listening on port {}",config.port.to_string());
    let string_public_key = String::from_utf8(public_key).unwrap();
    println!("Server : {}", string_public_key.clone());
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            let string_public_key = string_public_key.clone();
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        println!("{}: {:?}", addr, msg);
                        tx.send("Message recu".to_string());
                    }, 
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
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        thread::sleep(::std::time::Duration::from_millis(100));
    }
}
