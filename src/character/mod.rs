use json::{ object, JsonValue };

mod token_holder;
use token_holder::TokenHolder;

use crate::order::Order;

pub struct Character {
    pub name : String,
    tg : String,
    token : TokenHolder,
    info : JsonValue
}

impl Character {
    pub fn perfom_analysis(&mut self) -> Option<String> {
        let mut report = String::new();
        for data in self.get_orders().members() {
            let mut order = Order::from(data);
            order.assay();
            let result = order.render_assay_report(None);
            result.map(|s| report += &s);
        }
        if report.len() == 0 {
            Some(report)
        } else {
            None
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

    pub fn get_orders(&mut self) -> JsonValue {
        let url = format!("https://esi.evetech.net/v1/characters/{}/orders/", self.character_id());
        self.token.get(url.as_str())
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
