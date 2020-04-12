extern crate dotenv;

use std::time::Duration;
use std::thread::sleep;
use std::fs::{ File, read_to_string };
use std::env;
use json::JsonValue;
use std::io::prelude::*;
use itertools::Itertools;
use std::collections::HashMap;

mod character;
use character::Character;

mod requests;
mod order;
use order::{ Order, price::Price };

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

    let mut i = 0;

    let mut r = requests::Request::new();
    let mut data: Vec<order::Order> = r.get_whole_market(10000002).members().map(|itm| order::Order::from(itm)).collect();
    let mut result: HashMap<i64, Price> = HashMap::new();
    for order in data {
        match result.get_mut(&order.type_id) {
            Some(datum) => {
                *datum = *Price::min(datum, &order.price);
            }
            None => {
                result.insert(order.type_id, order.price);
            }
        }
    }

    let mut analyzed: Vec<(i64, Price)> = Vec::with_capacity(result.len());
    for (type_id, price) in result.iter() {
        analyzed.push((*type_id, *price));
    }
    analyzed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (i, o) in analyzed[0..300].iter().enumerate() {
        println!("{}: {} - {}", i, o.1, r.get_type(o.0)["name"].to_string())
    }
}
