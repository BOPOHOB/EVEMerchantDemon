use serde::{Serialize, Deserialize};

use crate::requests::Request;

pub mod overprint;
mod price;
use price::Price;
use overprint::Overprint;

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    is_buy_order: bool,
    #[serde(rename = "_id")]
    pub order_id: i64,
    price: Price,
    region_id: Option<i64>,
    type_id: i64,
    pub assay_result: Option<Overprint>,
}

impl Order {
    pub fn from_obsolete_json(data: &json::JsonValue) -> Self {
        Order {
            is_buy_order: false,
            order_id: -1,
            price: (&data["price"]).into(),
            type_id: data["type_id"].as_i64().unwrap(),
            region_id: None,
            assay_result: Some(
                Overprint{
                    position: data["position"].as_i64().unwrap(),
                    best_price: (&data["strongest_rival_price"]).into(),
                    analyzed: data["analyzed"].as_i64().unwrap(),
                }
            ),
        }
    }

    pub fn assay(&mut self) {
        self.assay_result = Some(Overprint::new());
        let mut is_self_founded = false;
        for rival in self.get_assay() {
            if rival.order_id == self.order_id {
                is_self_founded = true;
            } else {
                self.take_into_account(&rival);
            }
        }
        if !is_self_founded {
            println!("{} order wasn't found in location responce (critical inner logic fail) {}", self.order_id, self.get_type_name());
        }
    }

    fn get_assay(&self) -> Vec<Order> {
        Request::new().get_market_orders(
            self.region_id.expect("requere integer refion_id for user orders"),
            self.type_id,
            &self.order_type().to_string()
        ).members().map(|itm| Order::from(itm)).collect()
    }

    pub fn get_type_name(&self) -> String {
        Request::new().public_get(format!("https://esi.evetech.net/v3/universe/types/{}/", self.type_id).as_str())["name"].to_string()
    }

    fn order_type(&self) -> &str {
        match self.is_buy_order {
            true => "buy",
            false => "sell"
        }
    }

    pub fn take_into_account(&mut self, rival: &Order) {
        let overprint : &mut Overprint = self.assay_result.as_mut().unwrap();
        if rival.is_buy_order {
            overprint.best_price = *Price::max(&overprint.best_price, &rival.price);
            if self.price < rival.price {
                overprint.position += 1;
            }
        } else {
            overprint.best_price = *Price::min(&overprint.best_price, &rival.price);
            if self.price > rival.price {
                overprint.position += 1;
            }
        }
        overprint.analyzed += 1;
    }

    pub fn render_assay_report(&self, previous: Option<&Order>) -> Option<String> {
        let assay_result = self.assay_result.as_ref().expect("attempt render_assay_report before assay analyse");
        match previous {
            Some(prev) => {
                let prev_overprint = prev.assay_result.as_ref().unwrap();
                //if nothing changed
                if prev_overprint.position == assay_result.position && assay_result.analyzed == prev_overprint.analyzed {
                    return None;
                }
                //if price was changed by user
                if prev.price != self.price && assay_result.position == 0 {
                    return None;
                }
                //if user start to hold uniq order
                if prev_overprint.position == assay_result.position && prev_overprint.analyzed != 0 && assay_result.analyzed == 0 {
                    return Some(format!(
                        "Item *{}* (`{}`) don't have any assay",
                        self.get_type_name(),
                        self.price,
                    ));
                }
                //if user lose first position
                if prev_overprint.position != assay_result.position {
                    return Some(format!(
                        "{} ≫ {} ΔP = {} − {} = {}; {} *{}*",
                        prev_overprint.position + 1,
                        assay_result.position + 1,
                        Price::delta(&self.price, &assay_result.best_price),
                        self.price,
                        assay_result.best_price,
                        self.order_type(),
                        self.get_type_name(),
                    ));
                }
                None
            }
            None => {
                if assay_result.position != 0 {
                    return Some(format!(
                        "Order *{}* {} in `{}:{}` price {}",
                        self.get_type_name(),
                        self.order_type(),
                        assay_result.position + 1,
                        assay_result.analyzed + 1,
                        self.price,
                    ));
                }
                None
            }
        }
    }
}

impl From<&json::JsonValue> for Order {
    fn from(data: &json::JsonValue) -> Self {
        Order {
            is_buy_order: data["is_buy_order"].as_bool().expect("expect boolean is_buy_order in order"),
            order_id: data["order_id"].as_i64().expect("expect integer order_id in order"),
            price: data["price"].as_f64().expect("expect number price in order").into(),
            region_id: data["region_id"].as_i64(),
            type_id: data["type_id"].as_i64().expect("expect integer type_id in order"),
            assay_result: Some(Overprint::new()),
        }
    }
}
