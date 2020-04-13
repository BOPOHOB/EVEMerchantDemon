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
mod db;

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

fn read_characters() -> Vec<Character> {
    let mut result = db::DB::load_characters();
    if result.len() == 0 {
        println!("reread from file");
        result = read_auth_json_from_file().members().map(Character::from).collect();
    }
    result
}

fn main() {
    dotenv::dotenv().ok();

    let mut i = 0;

    loop {
        let mut characters = read_characters();
        for character in characters.iter_mut() {
            let report = character.perfom_analysis();
            report.map(|s| character.say(&s));
        }
        db::DB::save_characters(characters);
        let recall_timeout: Duration = Duration::new(5 * 60, 0);
        println!("sleep {}", i);
        sleep(recall_timeout);
        i += 1;
        println!("wakeup {}", i);
    }
}
