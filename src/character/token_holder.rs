use json::JsonValue;
use serde::{Serialize, Deserialize};
use time::{ OffsetDateTime, Duration };

use crate::requests::Request;

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenHolder {
    access_token: String,
    refresh_token: String,
    timestamp: i64,
}

impl TokenHolder {
    pub fn get(&mut self, url: &str) -> JsonValue {
        self.check_token();
        Request::new().character_get(url, self.access_token.as_str())
    }

    fn is_need_to_update_token(&self) -> bool {
        self.life_time() - OffsetDateTime::now() < Duration::new(20, 0)
    }

    fn life_time(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.timestamp)
    }

    fn set_life_time(&mut self, val: OffsetDateTime) {
        self.timestamp = val.timestamp();
    }

    fn check_token(&mut self) {
        if !self.is_need_to_update_token() {
            return;
        }
        let result = Request::new().public_post(
            "https://login.eveonline.com/oauth/token",
            &json::object!{
                grant_type: "refresh_token",
                refresh_token: self.refresh_token.as_str()
            }
        );
        self.access_token = result["access_token"].to_string();
        self.set_life_time(OffsetDateTime::now() + Duration::new(result["expires_in"].as_u32().expect("Login responce unexpected format").into(), 0));
    }
}
