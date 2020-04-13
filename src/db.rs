use std::collections::HashMap;
use mongodb::{
    Client,
    Database,
    Collection,
};

use crate::{
    order::Order,
    character::Character,
};

pub(crate) struct DB(Database);

impl DB {
    pub fn load_characters() -> Vec<Character> {
        let collection: mongodb::Collection = DB::new().collection("auth");
        collection.find(Some(bson::doc!{}), None)
            .expect("can't read auth from db")
            .map(|i| -> Character {
                bson::from_bson(bson::Bson::Document(i.expect("critical db inconsistancy"))).expect("critical db inconsistancy")
            })
            .collect()
    }

    pub fn save_characters(data: Vec<Character>) {
        let cllctn: Collection = DB::new().collection("auth");

        cllctn.delete_many(bson::doc!{}, None).expect("can't clear auth db");

        let data: Vec<bson::Document> = data.iter()
            .map(|i| {
                match bson::to_bson(&i).expect("can't serealize auth") {
                    bson::Bson::Document(doc) => doc,
                    _ => panic!("auth serealizer expect serde doc")
                }
            })
            .collect();
        cllctn.insert_many(data, None).expect("can't write auth to db");
    }

    pub fn load_orders(collection: &str) -> HashMap<i64, Order> {
        let cllctn: mongodb::Collection = DB::new().collection(collection);

        let mut result: HashMap<i64, Order> = HashMap::new();
        for doc in cllctn.find(Some(bson::doc!{}), None).expect("can't read from db") {
            let order: Order = bson::from_bson(bson::Bson::Document(doc.unwrap())).expect("reading error");
            result.insert(order.order_id, order);
        }
        result
    }

    pub fn save_orders(data: HashMap<i64, Order>, collection: &str) {
        let cllctn: Collection = DB::new().collection(collection);

        cllctn.delete_many(bson::doc!{}, None).expect("can't clear db");

        let data: Vec<bson::Document> = data.values()
            .map(|i| {
                match bson::to_bson(&i).expect("can't serealize") {
                    bson::Bson::Document(doc) => doc,
                    _ => panic!("expect serde doc")
                }
            })
            .collect();
        cllctn.insert_many(data, None).expect("can't write to db");
    }

    fn collection(&self, name: &str) -> Collection {
        self.0.collection(name)
    }

    fn new() -> Self {
        let client = Client::with_uri_str(std::env::var("MONGO_URL").expect("expect MONGO_URL environment variable").as_str()).expect("MONGO_URL is incorrect");
        DB(client.database(std::env::var("DB_NAME").expect("expect DB_NAME environment variable").as_str()))
    }
}