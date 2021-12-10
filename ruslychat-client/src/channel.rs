use std::io;

pub struct Channel {
    id: u32,
    name: String,
    description: String
}

impl Channel {
    pub fn insert(user_hash: String, name: String, description: String) {
        //TODO call API to add channel
    }

    pub fn close() {

    }
}

pub fn display_main_menu() -> u32 {
    let mut answer = String::from("1");

    while answer.eq("0") == false {
        println!("========================\n       Main Menu       \n========================");
        println!("1 : Open");
        println!("2 : New");
        println!("0 : Exit");

        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        match &*answer {
            "1" => {
                display_channel_menu();
            },
            "2" => {
                println!("Coming soon...");
                display_main_menu();
                break;
            },
            _ => (),
        }
    }

    0
}

fn display_channel_menu() {
    let mut answer = String::from("1");
    let channels = vec![
        Channel{ id: 10, name: "channel1".to_string(), description: "".to_string() },
        Channel{ id: 20, name: "channel2".to_string(), description: "".to_string() },
        Channel{ id: 30, name: "channel3".to_string(), description: "".to_string() },
    ];
    //TODO get list of channels from API

    while answer.eq("0") == false {
        let mut buff = String::new();

        println!("========================\n     Channel List      \n========================");
        for channel in &channels {
            println!("{} {}", channel.id, channel.name);
        }

        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        answer = buff.trim().to_string();

        println!("answer = {}", answer);

        for channel in &channels {
            if channel.id.to_string() == answer.to_string() {
                println!("{} {} {}", channel.id, channel.name, channel.description);
            }
        }
        //channels.iter().map(|channel| println!("{} {} {}", channel.id, channel.name, channel.description));
        //channels.into_iter().filter(|channel| channel.id == 20);
        /*for channel in channels.borrow() {
            //println!("{} {}", channel.id, channel.name);

            if channel.id.to_string() == answer.to_string() {
                println!("{} {} {}", channel.id, channel.name, channel.description);
            }
        }*/
    }
}