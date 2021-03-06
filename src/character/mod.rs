use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::{
    order::Order,
    requests::Request,
    db::DB,
};

mod token_holder;
use token_holder::TokenHolder;

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub name : String,
    tg : String,
    token : TokenHolder,
    #[serde(rename = "_id")]
    id: i64,
}

impl Character {
    pub fn perfom_analysis(&mut self) -> Option<String> {
        let mut report = String::new();
        let mut orders: HashMap<i64, Order> = match self.get_orders() {
            Ok(o) => o,
            Err(_) => {
                println!("can't get orders list for {} user because network issue", self.name);
                return None
            }
        };
        println!("{} analyze for {} orders", self.name, orders.len());
        let mut prev = self.load_prev_assay();
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
        DB::save_orders(data, self.mongo_collection_name().as_str())
    }

    fn load_prev_assay(&self) -> HashMap<i64, Order> {
        DB::load_orders(self.mongo_collection_name().as_str())
    }

    fn mongo_collection_name(&self) -> String {
        self.id.to_string()
    }

    pub fn get_orders(&mut self) -> Result<HashMap<i64, Order>, ()> {
        let url: String = format!("/v1/characters/{}/orders/", self.id);
        let mut result: HashMap<i64, Order> = HashMap::new();
        for order_data in self.token.get(&url)?.members() {
            let order = Order::from(order_data);
            result.insert(order.order_id, order);
        }
        Ok(result)
    }

    pub fn say(&self, message: &String) {
        if !cfg!(debug_assertions) {
            Request::new().say(self.tg.parse().unwrap(), &message.as_str());
        }
    }
}
