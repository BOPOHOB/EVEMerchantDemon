use json::JsonValue;
use std::fmt;
use std::fmt::{ Display };
use std::cmp::{ PartialOrd, PartialEq, Ordering };
use Ordering::{ Less, Equal, Greater };

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Price(f64);

impl Price {
    pub fn delta(lhs : &Price, rhs: &Price) -> Price {
        return (lhs.0 - rhs.0).abs().into()
    }

    pub fn min(lhs : Price, rhs : Price) -> Price {
        match lhs.partial_cmp(&rhs) {
            Some(result) => {
                match result {
                    Less | Equal => { lhs }
                    Greater => rhs
                }
            }
            None => {
                if lhs == lhs { lhs }
                else { rhs }
             }
        }
    }
    pub fn max(lhs : Price, rhs : Price) -> Price  {
        Price::min (rhs, lhs)
    }
}

impl Default for Price {
    fn default() -> Self {
        Price(f64::NAN)
    }
}

impl From<Price> for JsonValue {
    fn from(data: Price) -> Self {
        data.0.into()
    }
}

impl From<&JsonValue> for Price {
    fn from(data: &JsonValue) -> Self {
        Price(data.as_f64().expect("cost must be a float value"))
    }
}

impl From<f64> for Price {
    fn from(value: f64) -> Self {
        Price(value)
    }
}

impl Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0);
        Ok(())
    }
}