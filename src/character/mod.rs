use json::{ object, JsonValue };
use std::fs::{ File, read_to_string };
use std::io::prelude::*;

mod token_holder;
use token_holder::TokenHolder;

use crate::order::{ Order, OrderAnalyze };

pub struct Character {
    pub name : String,
    tg : String,
    token : TokenHolder,
    info : JsonValue
}

impl Character {
    pub fn perfom_analysis(&mut self) -> Option<String> {
        let mut report = String::new();
        let mut orders = self.get_orders();
        let mut i = 0usize;
        let prev = self.load_assay_file();
        for order in &mut orders {
            print!("{} ", i);
            order.assay();
            let result = order.render_assay_report(prev.get(i));
            result.map(|s| report += format!("{}\n", &s).as_str());
            i += 1;
        }
        self.save_assay_file(&orders);
        if report.len() != 0 {
            Some(report)
        } else {
            None
        }
    }

    fn save_assay_file(&self, data: &Vec<Order>) {
        let mut content = JsonValue::new_array();
        for order in data {
            content.push(JsonValue::from(&order.assay_result)).expect("inner logic fail");
        }
        let fname = self.assay_file_name();
        let mut file = File::create(&fname).expect(format!("Can't open file \"{}\" for write", &fname).as_str());
        file.write_all(content.pretty(2).as_bytes()).expect(format!("Can't save data to file \"{}\"", &fname).as_str());
    }

    fn load_assay_file(&self) -> Vec<OrderAnalyze> {
        match read_to_string(&self.assay_file_name()) {
            Ok(content) => {
                let data = json::parse(&content).expect(format!("Auth file \"{}\" isn't json", &self.assay_file_name()).as_str());
                let mut result = Vec::new();
                for datum in data.members() {
                    result.push(OrderAnalyze::from(datum));
                }
                result
            },
            Err(_) => {
                vec![]
            }
        }
    }

    fn assay_file_name(&self) -> String {
        format!("assay {}.json", self.name)
    }

    pub fn get_info(&mut self) -> &JsonValue {
        if self.info.is_null() {
            self.info = self.token.get("https://login.eveonline.com/oauth/verify/");
        }
        &self.info
    }

    pub fn character_id(&mut self) -> i64 {
        self.get_info()["CharacterID"].as_i64().expect("inner logic fail")
    }

    pub fn get_orders(&mut self) -> Vec<Order> {
        let url = format!("https://esi.evetech.net/v1/characters/{}/orders/", self.character_id());
        self.token.get(url.as_str()).members().map(|itm| Order::from(itm)).collect()
    }

    pub fn say(&self, message: &String) {
        println!("{}", message);
    }
}

impl From<&JsonValue> for Character {
    fn from(data: &JsonValue) -> Self {
        Character {
            name: data["character_name"].to_string(),
            tg: data["tg_id"].to_string(),
            token: TokenHolder::from(data),
            info: JsonValue::Null,
        }
    }
}

impl From<&Character> for JsonValue {
    fn from(data: &Character) -> Self {
        let mut result = object!{
            character_name: data.name.as_str(),
            tg_id: data.tg.as_str(),
        };
        for (key, value) in JsonValue::from(&data.token).entries() {
            result.insert(key, value.clone()).unwrap();
        }
        result
    }
}
