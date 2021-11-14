extern crate chrono;

mod init;
mod log;

use log::Logger;
use log::LogLevel;

fn main() {
    let config = init::check_init_file();
    println!("{}", config.port);
    println!("{}", config.logs_directory);
    println!("{}", config.database);
    println!("{}", config.user);
    println!("{}", config.passwd);

    let mut logger = Logger {
        path: config.logs_directory,
        log_file: "".to_string(),
        max_size: 10
    };

    logger.log("coucou".to_string(), LogLevel::INFO);
}
