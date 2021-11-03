mod init;

fn main() {
    let config = init::check_init_file();
    println!("{}", config.domain);
    println!("{}", config.port_dest);
    println!("{}", config.logs_directory);
}
