
use pwhash::sha512_crypt;
use std::io;

pub fn hash_generator(){
    println!("Password to hash : ");
    let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
    let password_to_hash = buff.trim().to_string();
    let hash_setup = "$6$salt";
    let password_hashed = sha512_crypt::hash_with(hash_setup, password_to_hash).unwrap();
    println!("Hashed password : {}",password_hashed);
    std::process::exit(0);

}
