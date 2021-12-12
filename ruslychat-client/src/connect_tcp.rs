use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::init;

const MSG_SIZE: usize = 32;

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
