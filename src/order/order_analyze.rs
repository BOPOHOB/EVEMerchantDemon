use super::price::Price;
use super::Order;

use json::JsonValue;

pub struct OrderAnalyze {
    pub position: i64,
    pub price: Price,
    pub strongest_rival_price: Price,
    pub analyzed: i64,
}

impl OrderAnalyze {
    pub fn new(price: Price) -> Self {
        OrderAnalyze {
            price: price,
            position: 0i64,
            strongest_rival_price: Price::default(),
            analyzed: 0i64,
        }
    }

    pub fn take_into_account(&mut self, rival: &Order) {
        if rival.is_buy_order {
            self.strongest_rival_price = *Price::max(&self.strongest_rival_price, &rival.price);
            if self.price < rival.price {
                self.position += 1;
            }
        } else {
            self.strongest_rival_price = *Price::min(&self.strongest_rival_price, &rival.price );
            if self.price > rival.price {
                self.position += 1;
            }
        }
        self.analyzed += 1;
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
            price: Price::from(&data["price"]),
            strongest_rival_price: Price::from(&data["strongest_rival_price"]),
            analyzed: data["analyzed"].as_i64().expect("expect integer analyzed in OrderAnalyze"),
        }
    }
}
