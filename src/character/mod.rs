use json::{ object, JsonValue };
use std::fs::{ File, read_to_string };
use std::io::prelude::*;
use std::collections::HashMap;

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
        let prev = self.load_assay_file();
        let mut prev_ids : HashMap<i64, i64> = HashMap::new();
        for (order_id, overprint) in &prev {
            prev_ids.insert(*order_id, overprint.type_id);
        }
        for order in &mut orders {
            order.assay();
            let result = order.render_assay_report(prev.get(&order.order_id));
            result.map(|s| report += format!("{}\n", &s).as_str());
            prev_ids.remove(&order.order_id);
        }
        let mut request = Request::new();
        for (_, type_id) in prev_ids {
            let item_name = request.get_type(type_id)["name"].to_string();
            report += format!("Order discharge *{}*\n", item_name).as_str();
        }
        self.save_assay_file(&orders);
        if report.len() != 0 {
            Some(report)
        } else {
            None
        }
    }

    fn save_assay_file(&self, data: &Vec<Order>) {
        let mut content = JsonValue::new_object();
        for order in data {
            content.insert(format!("{}", order.order_id).as_str(), JsonValue::from(&order.assay_result)).expect("inner logic fail");
        }
        let fname = self.assay_file_name();
        let mut file = File::create(&fname).expect(format!("Can't open file \"{}\" for write", &fname).as_str());
        file.write_all(content.pretty(2).as_bytes()).expect(format!("Can't save data to file \"{}\"", &fname).as_str());
    }

    fn load_assay_file(&self) -> HashMap<i64, Overprint> {
        match read_to_string(&self.assay_file_name()) {
            Ok(content) => {
                let data = json::parse(&content).expect(format!("Auth file \"{}\" isn't json", &self.assay_file_name()).as_str());
                let mut result : HashMap<i64, Overprint> = HashMap::new();
                for (key, datum) in data.entries() {
                    result.insert(
                        key.parse().unwrap(),
                        Overprint::from(datum)
                    );
                }
                result
            },
            Err(_) => {
                HashMap::new()
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
        Request::new().say(self.tg.parse().unwrap(), &message.as_str());
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
