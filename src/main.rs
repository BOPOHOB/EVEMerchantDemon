extern crate dotenv;

use std::{ time::Duration, thread::sleep };

mod character;
mod requests;
mod order;
mod db;

fn main() {
    dotenv::dotenv().ok();

    let mut i = 0;

    loop {
        let mut characters = db::DB::load_characters();
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
