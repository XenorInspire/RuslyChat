mod init;
mod connect_tcp;

fn main() {

    let config = init::check_init_file();
    connect_tcp::start_connection(config);

}
