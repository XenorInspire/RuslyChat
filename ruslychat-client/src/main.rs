use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

mod init;

fn main() {
    let config = init::check_init_file();
    // match TcpStream::connect("localhost:3333") {

    match TcpStream::connect(format!("{}:{}", config.domain, config.port_dest.to_string())) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port {}", config.port_dest.to_string());

            let msg = b"Hello!";

            stream.write(msg).unwrap();
            println!("Sent Hello, awaiting reply...");

            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == msg {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");

}
