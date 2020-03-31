use json::{ object, JsonValue };

mod token_holder;
use token_holder::TokenHolder;

pub struct Character {
    pub name : String,
    tg : String,
    token : TokenHolder,
}

impl Character {
    pub fn perfom_analysis(&self) {
        println!("perfon analysis for {}", self.name);
    }
}

impl From<&JsonValue> for Character {
    fn from(data: &JsonValue) -> Self {
        Character {
            name: data["character_name"].to_string(),
            tg: data["tg_id"].to_string(),
            token: TokenHolder::from(data),
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
