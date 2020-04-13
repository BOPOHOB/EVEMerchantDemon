use tokio::runtime::Runtime;
use reqwest::{
    RequestBuilder,
    Client,
    header::{
        HeaderMap,
        USER_AGENT,
        CONTENT_TYPE,
        HeaderValue
    }
};
use std::env;
use json::{ object, JsonValue};

static USER_AGENT_NAME : &'static str = "bopohob merchant monitor";

fn get_env_variable(key: &str)->String {
    env::var(key).expect(format!("Env variable \"{}\" is not declared", key).as_str())
}

pub struct Request {
    context: Runtime
}

impl Request {
    pub fn generic_headers() -> HeaderMap {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(4);
        headers.insert(USER_AGENT,  HeaderValue::from_static(USER_AGENT_NAME));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub fn new() -> Self {
        Request {
            context: Runtime::new().expect("Can't get runtime for application web requests")
        }
    }

    pub fn say(&mut self, chat_id: i64, text: &str) {
        let domen = get_env_variable("TELEGRAM_API_URL");
        let token = get_env_variable("TELOXIDE_TOKEN");
        let url = format!("{}/bot{}/sendMessage", domen, token);
        let body = object!{
            text: text,
            chat_id: chat_id,
            parse_mode: "Markdown"
        };

        let builder = Client::new().post(&url)
            .headers(Request::generic_headers())
            .body(body.dump());
        self.context.block_on(builder.send()).expect("telegram send message fail");
    }

    pub fn request_json(&mut self, builder: RequestBuilder) -> Result<JsonValue, ()> {
        let mut attempt = 0;
        loop {
            let responce = self.context.block_on(builder.try_clone().expect("stream body are not supplied").send());
            match responce {
                Ok(taked_responce) => {
                    let unwraped_responce = taked_responce;
                    let url = format!("{}", unwraped_responce.url());
                    if unwraped_responce.status() == reqwest::StatusCode::OK {
                        let text = self.context.block_on(unwraped_responce.text()).expect(format!("can't get responce text from \"{}\"", url).as_str());
                        return Ok(json::parse(text.as_str()).expect(format!("Responce from \"{}\" isn't json {}", url, text).as_str()));
                    }
                    println!("{}, {}", url, unwraped_responce.status());
                },
                Err(err) => println!("{}: can't take responce from \"{}\" {:?}", &attempt, err.url().unwrap(), err)
            };
            attempt += 1;
            if attempt == 10 {
                return Err(());
            }
        }
    }

    fn url(path: &String) -> String {
        format!("{}{}", get_env_variable("EVE_API_HOST"), path)
    }

    pub fn public_get(&mut self, path: &String) -> Result<JsonValue, ()> {
        let request = Client::new().get(Request::url(path).as_str())
            .headers(Request::generic_headers())
            .basic_auth(
                get_env_variable("EVE_CLIENT_ID").as_str(),
                Some(get_env_variable("EVE_CLIENT_SECRET").as_str())
            );
        self.request_json(request)
    }

    pub fn character_get(&mut self, path: &String, token: &str) -> Result<JsonValue, ()> {
        let request = Client::new().get(Request::url(path).as_str())
            .headers(Request::generic_headers())
            .bearer_auth(token);
        self.request_json(request)
    }

    pub fn get_market_orders(&mut self, market_id: i64, type_id: i64, order_type: &String) -> Result<JsonValue, ()> {
        let mut result : JsonValue = JsonValue::new_array();
        for i in 1..500 {
            let array = self.public_get(&format!("/v1/markets/{}/orders/?page={}&type_id={}&order_type={}", market_id, i, type_id, order_type))?;
            for note in array.members() {
                result.push(note.clone()).expect("result.push(note.clone())");
            }
            if array.len() != 1000 {
                return Ok(result);
            }
        }
        Ok(result)
    }
}
