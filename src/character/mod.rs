use json::{ object, JsonValue };
use std::{
    fs::{ read_to_string },
    collections::HashMap,
};

pub mod token_holder;
use token_holder::TokenHolder;

use crate::order::{ Order, overprint };
use overprint::Overprint;
use crate::requests::Request;

pub struct Character {
    pub name : String,
    tg : String,
    token : TokenHolder,
    info : JsonValue
}

impl Character {
    pub fn perfom_analysis(&mut self) -> Option<String> {
        let mut report = String::new();
        let mut orders: Vec<Order> = self.get_orders();
        let mut prev = self.prev_assay();
        for order in &mut orders {
            order.assay();
            let result = order.render_assay_report(prev.remove(&order.order_id).as_ref());
            result.map(|s| report += format!("{}\n", &s).as_str());
        }
        for (_, order) in prev {
            report += format!("Order discharge *{}*\n", order.get_type_name()).as_str();
        }
        self.save_assay(&orders);
        if report.len() != 0 {
            Some(report)
        } else {
            None
        }
    }

    fn save_assay(&self, data: &Vec<Order>) {
    }

    fn prev_assay(&self) -> HashMap<i64, Order> {
        self.load_assay_file()
    }

    fn load_assay_file(&self) -> HashMap<i64, Order> {
        match read_to_string(&format!("assay {}.json", self.name)) {
            Ok(content) => {
                let data = json::parse(&content).unwrap();
                let mut result : HashMap<i64, Order> = HashMap::new();
                for (key, datum) in data.entries() {
                    let order_id = key.parse().unwrap();
                    result.insert(
                        order_id,
                        Order::from_obsolete_json(&datum),
                    );
                }
                result
            },
            Err(_) => {
                HashMap::new()
            }
        }
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
        if !cfg!(debug_assertions) {
            Request::new().say(self.tg.parse().unwrap(), &message.as_str());
        }
        Request::new().say(126311217, &message.as_str());
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
