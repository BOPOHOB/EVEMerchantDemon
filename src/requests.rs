use tokio::runtime::Runtime;
use reqwest::header::{ HeaderMap, USER_AGENT, CONTENT_TYPE, HeaderValue };
use reqwest::{ RequestBuilder, Client };
use std::env;
use json::JsonValue;

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

    fn request_json(&mut self, builder: RequestBuilder) -> JsonValue {
        let responce = self.context.block_on(builder.send());
        let responce = match responce {
            Ok(taked_responce) => taked_responce,
            Err(err) => panic!("can't take responce from \"{}\"", err.url().unwrap())
        };
        let expectation = String::from(format!("Responce from \"{}\" isn't json", responce.url()));
        let text = self.context.block_on(responce.text()).expect(expectation.as_str());
        json::parse(text.as_str()).expect(expectation.as_str())
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
}
