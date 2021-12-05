use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

extern crate openssl;

use openssl::rsa::{Padding, Rsa};
use openssl::symm::Cipher;

use crate::init;

const MSG_SIZE: usize = 500;

pub fn start_connection(config: init::Config) {
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

    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == "!quit" || tx.send(msg).is_err() {
            break;
        }
    }
    println!("bye bye!");
}

pub fn key_exchange(config: init::Config) {
    let passphrase = "client_rust_by_example";

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
    //println!("Encrypted: {:?}", buf);

    let data = buf;

    // Decrypt with private key
    let rsa = Rsa::private_key_from_pem_passphrase(&private_key, passphrase.as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    let _ = rsa
        .private_decrypt(&data, &mut buf, Padding::PKCS1)
        .unwrap();
    //println!("Decrypted: {}", String::from_utf8(buf).unwrap());

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

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        println!("test");
        match client.read_exact(&mut buff) {
            Ok(_) => {
                
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("Server : {}", String::from_utf8(msg).expect("Error: key"));
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

    let string_public_key = String::from_utf8(public_key).unwrap();
    //println!("Client : {}", String::from_utf8(private_key).unwrap());
    println!("Client : {}", string_public_key.clone());
    tx.send(string_public_key);
}
