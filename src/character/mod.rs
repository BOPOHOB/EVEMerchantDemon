use json::{ object, JsonValue };
use std::{
    fs::{ read_to_string },
    collections::HashMap,
};
use mongodb::Client;

pub mod token_holder;
use token_holder::TokenHolder;
use crate::order::Order;
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
        let mut orders: HashMap<i64, Order> = self.get_orders();
        println!("{} analyze for {} orders", self.name, orders.len());
        let mut prev = self.prev_assay();
        for order in orders.values_mut() {
            order.assay();
            let result = order.render_assay_report(prev.remove(&order.order_id).as_ref());
            result.map(|s| report += format!("{}\n", &s).as_str());
        }
        for (_, order) in prev {
            report += format!("Order discharge {} *{}*\n", order.order_direction(), order.get_type_name()).as_str();
        }
        self.save_assay(orders);
        if report.len() != 0 {
            Some(report)
        } else {
            None
        }
    }

    fn save_assay(&self, data: HashMap<i64, Order>) {
        // Parse a connection string into an options struct.
        let client = Client::with_uri_str(std::env::var("MONGO_URL").expect("expect MONGO_URL environment variable").as_str()).expect("MONGO_URL is incorrect");

        // Get a handle to a collection in the database.
        let db = client.database(std::env::var("DB_NAME").expect("expect DB_NAME environment variable").as_str());
        let collection: mongodb::Collection = db.collection(self.mongo_collection_name().as_str());

        collection.delete_many(bson::doc!{}, None).expect("can't clear db");

        let data: Vec<bson::Document> = data.values()
            .map(|i| {
                match bson::to_bson(&i).expect("can't serealize") {
                    bson::Bson::Document(doc) => doc,
                    _ => panic!("expect serde doc")
                }
            })
            .collect();
        collection.insert_many(data, None).expect("can't write to db");
    }

    fn mongo_collection_name(&self) -> &String {
        &self.tg
    }

    fn prev_assay(&self) -> HashMap<i64, Order> {
        let client = Client::with_uri_str(std::env::var("MONGO_URL").expect("expect MONGO_URL environment variable").as_str()).expect("MONGO_URL is incorrect");
        let db = client.database(std::env::var("DB_NAME").expect("expect DB_NAME environment variable").as_str());
        let collection: mongodb::Collection = db.collection(self.mongo_collection_name().as_str());

        let mut result: HashMap<i64, Order> = HashMap::new();
        for doc in collection.find(Some(bson::doc!{}), None).expect("can't read from db") {
            let order: Order = bson::from_bson(bson::Bson::Document(doc.unwrap())).expect("reading error");
            result.insert(order.order_id, order);
        }
        if result.len() == 0 {
            result = self.load_assay_file();
            println!("read assay from file {}", result.len());
            result
        } else {
            println!("read assay from db {}", result.len());
            result
        }
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

    pub fn get_orders(&mut self) -> HashMap<i64, Order> {
        let url = format!("https://esi.evetech.net/v1/characters/{}/orders/", self.character_id());
        let mut result: HashMap<i64, Order> = HashMap::new();
        for order_data in self.token.get(url.as_str()).members() {
            let order = Order::from(order_data);
            result.insert(order.order_id, order);
        }
        result
    }

    pub fn say(&self, message: &String) {
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
