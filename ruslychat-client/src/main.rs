use std::io;

mod connect_tcp;
mod init;

fn main() {
    let config = init::check_init_file();

    //connect_tcp::start_connection(config);

    let mut config = init::check_init_file();
    let backup = config.clone();
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
            "1" => connect_tcp::key_exchange(config.clone()),
            "2" => {
                config = init::change_config_values(config);
                if config != backup {
                    init::create_new_config_file(init::CURRENT_CONFIG_FILE_MODE, config.clone())
                }
            }
            _ => (),
        }
    }
}
