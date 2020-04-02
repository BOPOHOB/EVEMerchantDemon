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
    let mut file = File::create(&auth_file_name).expect(format!("Can't open file \"{}\" for write", &auth_file_name).as_str());
    file.write_all(data.pretty(2).as_bytes()).expect(format!("Can't save data to file \"{}\"", auth_file_name).as_str());
}

fn main() {
    dotenv::dotenv().ok();

    println!("get market orders");
    let orders = requests::Request::new().get_market_orders(10000002);

    let mut auth_info: JsonValue = read_auth_json_from_file();
    for auth_datum in auth_info.members_mut() {
        let mut character : Character = Character::from(&*auth_datum);
        character.perfom_analysis();
        let modified = JsonValue::from(&character);
        *auth_datum = modified;
    }
    save_auth_json_to_file(&auth_info);
    let recall_timeout: Duration = Duration::new(0, 10);
    sleep(recall_timeout);
}
