use std::path::Path;

static CONFIG_FILE: &str = "config/config.ini";

pub fn display(){

    println!("{}", Path::new(CONFIG_FILE).exists());

}