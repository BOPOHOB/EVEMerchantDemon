extern crate dotenv;

use std::time::Duration;
use std::thread::sleep;
use std::fs::{ File, read_to_string };
use std::env;
use json::JsonValue;
use std::io::prelude::*;

mod character;
use character::Character;

mod requests;
mod order;

fn read_auth_json_from_file() -> JsonValue {
    let auth_file_name = env::var("AUTH_FILE").expect("expect AUTH_FILE environment variable");
    match read_to_string(&auth_file_name) {
        Ok(content) => {
            json::parse(&content).expect(format!("Auth file \"{}\" isn't json", auth_file_name).as_str())
        },
        Err(_) => {
            println!("Can't open auth file \"{}\" (no one pilot pass the authorisation)", auth_file_name);
            JsonValue::new_array()
        }
    }
}

fn save_auth_json_to_file(data: &JsonValue) {
    if data.len() == 0 {
        return;
    }
    let auth_file_name = env::var("AUTH_FILE").expect("expect AUTH_FILE environment variable");
    save_json_to_file(data, auth_file_name.as_str());
}

fn save_json_to_file(data: &JsonValue, fname: &str) {
    let mut file = File::create(fname).expect(format!("Can't open file \"{}\" for write", fname).as_str());
    file.write_all(data.pretty(2).as_bytes()).expect(format!("Can't save data to file \"{}\"", fname).as_str());
}

fn main() {
    dotenv::dotenv().ok();

    let mut auth_info: JsonValue = read_auth_json_from_file();
    let mut i = 0;
    loop {
        for auth_datum in auth_info.members_mut() {
            let mut character : Character = Character::from(&*auth_datum);
            let report = character.perfom_analysis();
            report.map(|s| character.say(&s));
        }
        save_auth_json_to_file(&auth_info);
        let recall_timeout: Duration = Duration::new(5, 0);
        println!("sleep {}", i);
        sleep(recall_timeout);
        i += 1;
        println!("wakeup {}", i);
    }
}
