use json::JsonValue;
use serde::{Serialize, Deserialize};
use time::{ OffsetDateTime, Duration };
use reqwest::Client;
use std::env;

use crate::requests::Request;

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenHolder {
    access_token: String,
    refresh_token: String,
    timestamp: i64,
}

impl TokenHolder {
    pub fn get(&mut self, url: &String) -> Result<JsonValue, ()> {
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

    fn post_token_request(&self) -> JsonValue {
        let request = Client::new().post(&String::from(env::var("EVE_AUTH_URL").expect("Env variable EVE_AUTH_URL is not declared")))
            .headers(Request::generic_headers())
            .basic_auth(
                env::var("EVE_CLIENT_ID").expect("Env variable EVE_CLIENT_ID is not declared").as_str(),
                Some(env::var("EVE_CLIENT_SECRET").expect("Env variable EVE_CLIENT_SECRET is not declared").as_str())
            )
            .body(json::object!{
                grant_type: "refresh_token",
                refresh_token: self.refresh_token.as_str()
            }.dump());
        Request::new().request_json(request).expect("Critical network error")
    }

    fn check_token(&mut self) {
        if !self.is_need_to_update_token() {
            return;
        }
        let result = self.post_token_request();
        self.access_token = result["access_token"].to_string();
        self.set_life_time(OffsetDateTime::now() + Duration::new(result["expires_in"].as_u32().expect("Login responce unexpected format").into(), 0));
    }
}
