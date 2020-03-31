use json::{ object, JsonValue };
use time::{ OffsetDateTime, Duration };
use std::env;
use tokio::runtime::Runtime;

fn get_env_variable(key: &str)->String {
    env::var(key).expect(format!("Env variable \"{}\" is not declared", key).as_str())
}

pub struct TokenHolder {
    access_token: String,
    refresh_token: String,
    life_time: OffsetDateTime,
}

impl TokenHolder {
    fn post_request(&self, authorization: &str, url: &str, body : &JsonValue) -> JsonValue {
        let mut runtime = Runtime::new().expect("Login request get tokio runtime");
        let responce = runtime.block_on(reqwest::Client::new().post(url)
            .header("Authorization", authorization)
            .header("User-Agent", "bopohob merchant monitor")
            .header("Content-Type", "application/json")
            .body(body.dump())
            .send()).expect("Login attempt");
        let responce_text = runtime.block_on(responce.text()).expect("Login attempt responce body");
        json::parse(responce_text.as_str()).expect("Login responce json parse")
    }

    fn get_request(&self, url: &str) -> JsonValue {
        let mut runtime = Runtime::new().expect("Login request get tokio runtime");
        let responce = runtime.block_on(reqwest::Client::new().get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "bopohob merchant monitor")
            .header("Content-Type", "application/json")
            .send()).expect("Login attempt");
        let responce_text = runtime.block_on(responce.text()).expect("Login attempt responce body");
        json::parse(responce_text.as_str()).expect("Login responce json parse")
    }

    fn is_need_to_update_token(&self) -> bool {
        self.life_time - OffsetDateTime::now() > Duration::new(20, 0)
    }

    fn check_token(&mut self) {
        if self.is_need_to_update_token() {
            return;
        }
        let combo =  base64::encode(String::from(format!("{}:{}", get_env_variable("EVE_CLIENT_ID"), get_env_variable("EVE_CLIENT_SECRET"))));
        let result = self.post_request(
            format!("Basic {}", combo).as_str(),
            "https://login.eveonline.com/oauth/token",
            &json::object!{
                grant_type: "refresh_token",
                refresh_token: self.refresh_token.as_str()
            }
        );
        self.access_token = result["access_token"].to_string();
        self.life_time = OffsetDateTime::now() + Duration::new(result["expires_in"].as_u32().expect("Login responce unexpected format").into(), 0);
    }

    pub fn post(&mut self, url: &str, body: &JsonValue) -> JsonValue {
        self.check_token();
        self.post_request(
            format!("Bearer {}", self.access_token).as_str(),
            url,
            body
        )
    }

    pub fn get(&mut self, url: &str) -> JsonValue {
        self.check_token();
        self.get_request(
            url
        )
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
