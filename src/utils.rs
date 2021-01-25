use crate::error as err;
use lazy_static::lazy_static;
use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde_json::Value;
use snafu::ResultExt;
use url::Url;

#[macro_export]
macro_rules! value_to_string {
    ($v: expr) => {
        match $v {
            serde_json::Value::String(ref s) => Some(s.to_owned()),
            _ => None,
        }
    };
    ($v: expr, $or: expr) => {
        match $v {
            serde_json::Value::String(ref s) => Some(s.to_owned()),
            _ => $crate::value_to_string!($or),
        }
    };
}
#[macro_export]
macro_rules! hdmap {
    () => {
        reqwest::header::HeaderMap::new()
    };
    ($($key: expr => $value: expr),+ $(,)?) => {
        {
            use std::convert::TryInto;
            const CAP: usize = sugars::count!($($key),*);
            let mut map = reqwest::header::HeaderMap::with_capacity(CAP);
            $(map.insert($key, $value.try_into().unwrap());)+
            map
        }
    };
}

lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new().gzip(true).build().unwrap();
    pub static ref UA: HeaderValue = HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36");
}

#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
    header: HeaderMap,
}

impl Client {
    fn with_details(inner: reqwest::Client, header: HeaderMap) -> Self {
        Self { inner, header }
    }
    pub fn new() -> Self {
        Self::with_details(CLIENT.clone(), hdmap! { header::USER_AGENT => UA.clone() })
    }
    pub fn with_header(header: HeaderMap) -> Self {
        Self::with_details(CLIENT.clone(), header)
    }
    pub async fn send_json_request(&self, url: Url) -> Result<Value, err::Error> {
        Ok(self
            .inner
            .get(url.clone())
            .headers(self.header.clone())
            .send()
            .await
            .context(err::NetworkError { url })?
            .json()
            .await?)
    }
    pub fn client(&self) -> &reqwest::Client {
        &self.inner
    }
    pub async fn post_form_json<T: serde::Serialize>(
        &self,
        url: Url,
        form: &T,
    ) -> Result<Value, err::Error> {
        Ok(self
            .inner
            .post(url.clone())
            .headers(self.header.clone())
            .form(form)
            .send()
            .await
            .context(err::NetworkError { url })?
            .json::<Value>()
            .await?)
    }
    pub async fn post_json(&self, url: Url) -> Result<Value, err::Error> {
        Ok(self
            .inner
            .post(url.clone())
            .headers(self.header.clone())
            .send()
            .await
            .context(err::NetworkError { url })?
            .json::<Value>()
            .await?)
    }
}
