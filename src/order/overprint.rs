use super::price::Price;
use super::Order;

use json::JsonValue;

pub struct Overprint {
    pub position: i64,
    pub price: Price,
    pub strongest_rival_price: Price,
    pub analyzed: i64,
    pub type_id: i64,
}

impl Overprint {
    pub fn new(price: Price, type_id: i64) -> Self {
        Overprint {
            price: price,
            position: 0i64,
            strongest_rival_price: Price::default(),
            analyzed: 0i64,
            type_id: type_id,
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

impl From<&Overprint> for JsonValue {
    fn from(data: &Overprint) -> Self {
        json::object! {
            position: data.position,
            price: data.price,
            strongest_rival_price: data.strongest_rival_price,
            analyzed: data.analyzed,
            type_id: data.type_id,
        }
    }
}

impl From<&JsonValue> for Overprint {
    fn from(data: &JsonValue) -> Self {
        Overprint{
            position: data["position"].as_i64().expect("expect integer position in Overprint"),
            price: Price::from(&data["price"]),
            strongest_rival_price: Price::from(&data["strongest_rival_price"]),
            analyzed: data["analyzed"].as_i64().expect("expect integer analyzed in Overprint"),
            type_id: data["type_id"].as_i64().expect("expect integer type_id in Overprint"),
        }
    }
}