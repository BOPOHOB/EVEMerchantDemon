use tokio::runtime::Runtime;
use reqwest::header::{ HeaderMap, USER_AGENT, CONTENT_TYPE, HeaderValue };
use reqwest::{ get, RequestBuilder, Client };
use std::env;
use json::JsonValue;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

static USER_AGENT_NAME : &'static str = "bopohob merchant monitor";

fn get_env_variable(key: &str)->String {
    env::var(key).expect(format!("Env variable \"{}\" is not declared", key).as_str())
}

fn generic_headers() -> HeaderMap {
    let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(4);
    headers.insert(USER_AGENT,  HeaderValue::from_static(USER_AGENT_NAME));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

pub struct Request {
    context: Runtime
}

impl Request {
    pub fn new() -> Self {
        Request {
            context: Runtime::new().expect("Can't get runtime for application web requests")
        }
    }

    pub fn say(&mut self, chat_id: i64, text: &str) {
        let domen = get_env_variable("TELEGRAM_API_URL");
        let token = get_env_variable("TELOXIDE_TOKEN");
        let url = format!("{}/bot{}/sendMessage?chat_id={}&text={}", domen, token, chat_id, percent_encode(text.as_bytes(), NON_ALPHANUMERIC));

        self.context.block_on(get(url.as_str())).expect(format!("can't say to telegram {}: {}",chat_id, text).as_str());
    }

    fn request_json(&mut self, builder: RequestBuilder) -> JsonValue {
        loop {
            let responce = self.context.block_on(builder.try_clone().expect("stream body are not supplied").send());
            let unwraped_responce = match responce {
                Ok(taked_responce) => taked_responce,
                Err(err) => panic!("can't take responce from \"{}\" {:?}", err.url().unwrap(), err)
            };
            let url = format!("{}", unwraped_responce.url());
            if unwraped_responce.status() == reqwest::StatusCode::OK {
                let text = self.context.block_on(unwraped_responce.text()).expect(format!("can't get responce text from \"{}\"", url).as_str());
                return json::parse(text.as_str()).expect(format!("Responce from \"{}\" isn't json {}", url, text).as_str());
            }
            println!("{}, {}", url, unwraped_responce.status());
        }
    }

    pub fn public_get(&mut self, url: &str) -> JsonValue {
        let request = Client::new().get(url)
            .headers(generic_headers())
            .basic_auth(
                get_env_variable("EVE_CLIENT_ID").as_str(),
                Some(get_env_variable("EVE_CLIENT_SECRET").as_str())
            );
        self.request_json(request)
    }

    pub fn public_post(&mut self, url: &str, body: &JsonValue) -> JsonValue {
        let request = Client::new().post(url)
            .headers(generic_headers())
            .basic_auth(
                get_env_variable("EVE_CLIENT_ID").as_str(),
                Some(get_env_variable("EVE_CLIENT_SECRET").as_str())
            )
            .body(body.dump());
        self.request_json(request)
    }

    pub fn character_get(&mut self, url: &str, token: &str) -> JsonValue {
        let request = Client::new().get(url)
            .headers(generic_headers())
            .bearer_auth(token);
        self.request_json(request)
    }

    pub fn character_post(&mut self, url: &str, token: &str, body: &JsonValue) -> JsonValue {
        let request = Client::new().post(url)
            .headers(generic_headers())
            .bearer_auth(token)
            .body(body.dump());
        self.request_json(request)
    }

    pub fn get_market_orders(&mut self, market_id: i64, type_id: i64, order_type: &String) -> JsonValue {
        let mut result : JsonValue = JsonValue::new_array();
        for i in 1..43 {
            let array = self.public_get(format!("https://esi.evetech.net/v1/markets/{}/orders/?page={}&type_id={}&order_type={}", market_id, i, type_id, order_type).as_str());
            for note in array.members() {
                result.push(note.clone()).expect("result.push(note.clone())");
            }
            if array.len() != 1000 {
                return result;
            }
        }
        result
    }

    pub fn get_type(&mut self, id: i64) -> JsonValue {
        self.public_get(format!("https://esi.evetech.net/v3/universe/types/{}/", id).as_str())
    }
}
