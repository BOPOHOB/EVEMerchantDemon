use json::JsonValue;
use std::{
    fmt::{ self, Display },
    cmp::{ PartialOrd, PartialEq, Ordering },
};
use Ordering::{ Less, Equal, Greater };
use serde::{Serialize, Deserialize};

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Price(f64);

impl Price {
    pub fn delta(lhs : &Price, rhs: &Price) -> Price {
        return (lhs.0 - rhs.0).abs().into()
    }

    fn take_cmp<'a>(o: Option<Ordering>, lhs : &'a Price, rhs : &'a Price) -> &'a Price {
        match o {
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

    pub fn min<'a>(lhs : &'a Price, rhs : &'a Price) -> &'a Price {
        Price::take_cmp(lhs.partial_cmp(rhs), lhs, rhs)
    }
    pub fn max<'a>(lhs : &'a Price, rhs : &'a Price) -> &'a Price  {
        Price::take_cmp(rhs.partial_cmp(lhs), lhs, rhs)
    }

    pub fn new(value: f64) -> Price {
        Price(value)
    }
}

impl Default for Price {
    fn default() -> Self {
        Price(f64::NAN)
    }
}

impl From<&JsonValue> for Price {
    fn from(data: &JsonValue) -> Self {
        match *data {
            JsonValue::Null => { Price(f64::NAN) }
            JsonValue::Number(value) => { Price(value.into()) }
            _ => { panic!("Price must be a float value") }
        }
    }
}

impl From<Price> for JsonValue {
    fn from(data: Price) -> Self {
        data.0.into()
    }
}

impl From<f64> for Price {
    fn from(value: f64) -> Self {
        Price(value)
    }
}

impl Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 1e9 {
            write!(f, "{}B", self.0 / 1e9)?;
        } else if self.0 >= 1e6 {
            write!(f, "{}M", self.0 / 1e6)?;
        } else {
            write!(f, "{}", self.0)?;
        }
        Ok(())
    }
}
