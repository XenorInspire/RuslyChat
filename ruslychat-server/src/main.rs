mod init;
mod tcp_listening;

fn main() {
    let config = init::check_init_file();
    tcp_listening::start_listening(config);
}
