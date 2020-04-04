use crate::requests::Request;

use json::JsonValue;

pub struct Order {
    is_buy_order: bool,
    order_id: i64,
    price: f64,
    region_id: Option<i64>,
    type_id: i64,
    assay_result: OrderAnalyze,
}

pub struct OrderAnalyze {
    position: i64,
    price: f64,
    strongest_rival_price: f64,
    analyzed: i64,
}

impl OrderAnalyze {
    fn new(price: f64) -> Self {
        OrderAnalyze {
            price: price,
            position: 0i64,
            strongest_rival_price: 0f64,
            analyzed: 0i64,
        }
    }

    fn take_into_account(&mut self, rival: &Order) {
        if rival.is_buy_order {
            self.strongest_rival_price = if self.strongest_rival_price > rival.price {self.strongest_rival_price } else { rival.price };
            if self.price < rival.price {
                self.position += 1;
            }
        } else {
            self.strongest_rival_price = if self.strongest_rival_price < rival.price {self.strongest_rival_price } else { rival.price };
            if self.price > rival.price {
                self.position += 1;
            }
        }
        self.analyzed += 1;
    }
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
        assert!(is_self_founded, "{} order wasn't found in location responce (critical inner logic fail)", self.order_id);
    }

    fn get_assay(&self) -> Vec<Order> {
        Request::new().get_market_orders(self.region_id.expect("requere refion_id for user orders"), self.type_id, &self.order_type().to_string()).members().map(|itm| Order::from(itm)).collect()
    }

    fn order_type(&self) -> &str {
        match self.is_buy_order {
            true => "buy",
            false => "sell"
        }
    }

    pub fn render_assay_report(&self, previous: Option<&OrderAnalyze>) -> Option<String> {
        match previous {
            Some(prev) => {
                //if user change price after previous analyze
                if prev.price != self.price {
                    return None;
                }
                //if nothing changed
                if prev.position == self.assay_result.position && self.assay_result.analyzed == prev.analyzed {
                    return None;
                }
                //if user start to hold uniq order
                if prev.position == self.assay_result.position && prev.analyzed != 0 && self.assay_result.analyzed == 0 {
                    return Some(format!(
                        "From now you don't have any assay for {} with price {}",
                        self.type_id,
                        self.price,
                    ));
                }
                //if user lose first position
                if prev.position == 0 && self.assay_result.position != 0 {
                    return Some(format!(
                        "Somebody push order {} from first position by the cost {} (eour prive {})",
                        self.type_id,
                        self.assay_result.strongest_rival_price,
                        self.price,
                    ));
                }
                None
            }
            None => {
                Some(format!(
                    "Order {} in {} of {} with price {}",
                    self.type_id,
                    self.assay_result.position,
                    self.assay_result.analyzed,
                    self.price,
                ))
            }
        }
    }
}

impl From<&JsonValue> for Order {
    fn from(data: &JsonValue) -> Self {
        Order {
            is_buy_order: data["is_buy_order"].as_bool().expect("expect boolean is_buy_order in order"),
            order_id: data["order_id"].as_i64().expect("expect integer order_id in order"),
            price: data["price"].as_f64().expect("expect number price in order"),
            region_id: data["region_id"].as_i64(),
            type_id: data["type_id"].as_i64().expect("expect integer type_id in order"),
            assay_result: OrderAnalyze::new(0f64),
        }
    }
}
