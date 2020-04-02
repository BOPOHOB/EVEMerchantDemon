use json::{ object, JsonValue };
use time::{ OffsetDateTime, Duration };

use crate::requests::Request;

pub struct TokenHolder {
    access_token: String,
    refresh_token: String,
    life_time: OffsetDateTime,
}

impl TokenHolder {
    pub fn post(&mut self, url: &str, body : &JsonValue) -> JsonValue {
        self.check_token();
        Request::new().character_post(url, self.access_token.as_str(), body)
    }

    pub fn get(&mut self, url: &str) -> JsonValue {
        self.check_token();
        Request::new().character_get(url, self.access_token.as_str())
    }

    fn is_need_to_update_token(&self) -> bool {
        self.life_time - OffsetDateTime::now() > Duration::new(20, 0)
    }

    fn check_token(&mut self) {
        if self.is_need_to_update_token() {
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
        self.life_time = OffsetDateTime::now() + Duration::new(result["expires_in"].as_u32().expect("Login responce unexpected format").into(), 0);
    }
}

impl From<&JsonValue> for TokenHolder {
    fn from(data: &JsonValue) -> Self {
        let mut holder = TokenHolder {
            refresh_token: data["refresh_token"].to_string(),
            life_time: OffsetDateTime::from_unix_timestamp(data["expires_in"].as_i64().expect("auth expires_in shoud be a timestamp")),
            access_token: data["access_token"].to_string()
        };
        holder.check_token();
        holder
    }
}

impl From<&TokenHolder> for JsonValue {
    fn from(data: &TokenHolder) -> Self {
        object!{
            refresh_token: data.refresh_token.as_str(),
            expires_in: data.life_time.timestamp(),
            access_token: data.access_token.as_str(),
        }
    }
}
