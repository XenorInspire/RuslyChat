use std::io;

mod init;
mod connect_tcp;

fn main() {

    let config = init::check_init_file();

    connect_tcp::start_connection(config);

    let mut answer = String::from("1");

    while answer.eq("0") == false {
        println!("========================\nWelcome to RuslyChat !\n========================");
        println!("1 : Log in");
        println!("2 : Manage settings");
        println!("0 : Exit");

        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();
    }
}
