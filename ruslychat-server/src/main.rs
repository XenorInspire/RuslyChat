mod init;

fn main() {
    let config = init::check_init_file();
    println!("{}", config.port);
    println!("{}", config.logs_directory);
    println!("{}", config.database);
    println!("{}", config.user);
    println!("{}", config.passwd);
}
