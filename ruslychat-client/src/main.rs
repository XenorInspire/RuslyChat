use std::io;

mod init;

fn main() {
    let mut config = init::check_init_file();
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
            "2" => config = init::change_config_values(config),
            _ => (),
        }
    }
}
