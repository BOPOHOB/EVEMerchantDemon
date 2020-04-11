use serde::{Serialize, Deserialize};
use super::price::Price;

#[derive(Serialize, Deserialize, Debug)]
pub struct Overprint {
    pub position: i64,
    pub best_price: Price,
    pub analyzed: i64,
}

impl Overprint {
    pub fn new() -> Self {
        Overprint {
            position: 0,
            best_price: f64::NAN.into(),
            analyzed: 0
        }
    }
}
