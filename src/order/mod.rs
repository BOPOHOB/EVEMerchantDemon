use crate::requests::Request;

pub mod overprint;
mod price;
use price::Price;
use overprint::Overprint;

use json::JsonValue;


pub struct Order {
    is_buy_order: bool,
    pub order_id: i64,
    price: Price,
    region_id: Option<i64>,
    type_id: i64,
    pub assay_result: Overprint,
}

impl Order {
    pub fn assay(&mut self) {
        let mut is_self_founded = false;
        for rival in self.get_assay() {
            if rival.order_id == self.order_id {
                is_self_founded = true;
            } else {
                self.assay_result.take_into_account(&rival);
            }
        }
        if !is_self_founded {
            let item_name = Request::new().get_type(self.type_id)["name"].to_string();
            println!("{} order wasn't found in location responce (critical inner logic fail) {}", self.order_id, item_name);
        }
    }

    fn get_assay(&self) -> Vec<Order> {
        Request::new().get_market_orders(
            self.region_id.expect("requere integer refion_id for user orders"),
            self.type_id,
            &self.order_type().to_string()
        ).members().map(|itm| Order::from(itm)).collect()
    }

    fn order_type(&self) -> &str {
        match self.is_buy_order {
            true => "buy",
            false => "sell"
        }
    }

    pub fn render_assay_report(&self, previous: Option<&Overprint>) -> Option<String> {
        match previous {
            Some(prev) => {
                //if nothing changed
                if prev.position == self.assay_result.position && self.assay_result.analyzed == prev.analyzed {
                    return None;
                }
                //if price was changed by user
                if prev.price != self.price && self.assay_result.position == 0 {
                    return None;
                }
                //if user start to hold uniq order
                if prev.position == self.assay_result.position && prev.analyzed != 0 && self.assay_result.analyzed == 0 {
                    let item_name = Request::new().get_type(self.type_id)["name"].to_string();
                    return Some(format!(
                        "Item *{}* (`{}`) don't have any assay",
                        item_name,
                        self.price,
                    ));
                }
                //if user lose first position
                if prev.position != self.assay_result.position {
                    let item_name = Request::new().get_type(self.type_id)["name"].to_string();
                    return Some(format!(
                        "Position changed `{}>>{}`; delta: {} ({} -- {}); {} *{}*",
                        prev.position + 1,
                        self.assay_result.position + 1,
                        Price::delta(&self.price, &self.assay_result.strongest_rival_price),
                        self.price,
                        self.assay_result.strongest_rival_price,
                        self.order_type(),
                        item_name,
                    ));
                }
                None
            }
            None => {
                if self.assay_result.position != 0 {
                    let item_name = Request::new().get_type(self.type_id)["name"].to_string();
                    return Some(format!(
                        "Order *{}* {} in `{}:{}` price {}",
                        item_name,
                        self.order_type(),
                        self.assay_result.position + 1,
                        self.assay_result.analyzed + 1,
                        self.price,
                    ));
                }
                None
            }
        }
    }
}

impl From<&JsonValue> for Order {
    fn from(data: &JsonValue) -> Self {
        let price: Price = data["price"].as_f64().expect("expect number price in order").into();
        let type_id = data["type_id"].as_i64().expect("expect integer type_id in order");
        Order {
            is_buy_order: data["is_buy_order"].as_bool().expect("expect boolean is_buy_order in order"),
            order_id: data["order_id"].as_i64().expect("expect integer order_id in order"),
            price: price,
            region_id: data["region_id"].as_i64(),
            type_id: type_id,
            assay_result: Overprint::new(price, type_id),
        }
    }
}
