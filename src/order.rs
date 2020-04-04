use crate::requests::Request;

use json::JsonValue;

pub struct Order {
    is_buy_order: bool,
    pub order_id: i64,
    price: f64,
    region_id: Option<i64>,
    type_id: i64,
    pub assay_result: OrderAnalyze,
}

pub struct OrderAnalyze {
    position: i64,
    price: f64,
    strongest_rival_price: f64,
    analyzed: i64,
}

impl OrderAnalyze {
    pub fn new(price: f64) -> Self {
        OrderAnalyze {
            price: price,
            position: 0i64,
            strongest_rival_price: 0f64,
            analyzed: 0i64,
        }
    }

    fn take_into_account(&mut self, rival: &Order) {
        if rival.is_buy_order {
            self.strongest_rival_price = if self.strongest_rival_price > rival.price { self.strongest_rival_price } else { rival.price };
            if self.price < rival.price {
                self.position += 1;
            }
        } else {
            self.strongest_rival_price = if self.strongest_rival_price < rival.price && self.strongest_rival_price != 0f64 {self.strongest_rival_price } else { rival.price };
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

    pub fn render_assay_report(&self, previous: Option<&OrderAnalyze>) -> Option<String> {
        match previous {
            Some(prev) => {
                //println!("prev {} {} {} {}", prev.position, prev.analyzed, self.assay_result.position, self.assay_result.analyzed);
                //if nothing changed
                if prev.position == self.assay_result.position && self.assay_result.analyzed == prev.analyzed {
                    return None;
                }
                //if user start to hold uniq order
                if prev.position == self.assay_result.position && prev.analyzed != 0 && self.assay_result.analyzed == 0 {
                    return None;
                    /*let item_name = Request::new().get_type(self.type_id)["name"].to_string();
                    return Some(format!(
                        "From now you don't have any assay for {} ({}) with price {}",
                        self.type_id,
                        item_name,
                        self.price,
                    ));*/
                }
                //if user lose first position
                if prev.position != self.assay_result.position {
                    let item_name = Request::new().get_type(self.type_id)["name"].to_string();
                    return Some(format!(
                        "Position changed {} >> {} of {}; delta: {} ({} -- {}); {} {} ({})",
                        prev.position + 1,
                        self.assay_result.position + 1,
                        self.assay_result.analyzed + 1,
                        (self.price - self.assay_result.strongest_rival_price).abs(),
                        self.price,
                        self.assay_result.strongest_rival_price,
                        self.order_type(),
                        self.type_id,
                        item_name,
                    ));
                }
                None
            }
            None => {
                println!("pub {}", self.assay_result.position);
                if self.assay_result.position != 0 {
                    let item_name = Request::new().get_type(self.type_id)["name"].to_string();
                    return Some(format!(
                        "Order {} ({}) {} in {} of {} with price {}",
                        self.type_id,
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
        let price = data["price"].as_f64().expect("expect number price in order");
        Order {
            is_buy_order: data["is_buy_order"].as_bool().expect("expect boolean is_buy_order in order"),
            order_id: data["order_id"].as_i64().expect("expect integer order_id in order"),
            price: price,
            region_id: data["region_id"].as_i64(),
            type_id: data["type_id"].as_i64().expect("expect integer type_id in order"),
            assay_result: OrderAnalyze::new(price),
        }
    }
}

impl From<&OrderAnalyze> for JsonValue {
    fn from(data: &OrderAnalyze) -> Self {
        json::object! {
            position: data.position,
            price: data.price,
            strongest_rival_price: data.strongest_rival_price,
            analyzed: data.analyzed,
        }
    }
}

impl From<&JsonValue> for OrderAnalyze {
    fn from(data: &JsonValue) -> Self {
        OrderAnalyze{
            position: data["position"].as_i64().expect("expect integer position in OrderAnalyze"),
            price: data["price"].as_f64().expect("expect float price in OrderAnalyze"),
            strongest_rival_price: data["strongest_rival_price"].as_f64().expect("expect float strongest_rival_price in OrderAnalyze"),
            analyzed: data["analyzed"].as_i64().expect("expect integer analyzed in OrderAnalyze"),
        }
    }
}
