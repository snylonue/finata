use crate::error as err;
use once_cell::sync::Lazy;
use reqwest::{
    header,
    header::{HeaderMap, HeaderValue},
};
use serde_json::Value;
use snafu::{ResultExt, Snafu};
use std::path::Path;
use url::Url;

pub static CLIENT: Lazy<reqwest::Client> =
    Lazy::new(|| reqwest::ClientBuilder::new().gzip(true).build().unwrap());
pub static UA: Lazy<HeaderValue> = Lazy::new(|| {
    HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36")
});

#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
    header: HeaderMap,
}

#[derive(Debug, Snafu)]
pub enum ClientError {
    #[snafu(context(false))]
    IoError { source: std::io::Error },
    #[snafu(context(false))]
    InvalidNetscapeCookie { source: nescookie::error::Error },
    #[snafu(context(false))]
    InvalidCookie { source: header::InvalidHeaderValue },
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
    pub fn push_cookie(&mut self, cookie: &str) -> Result<(), ClientError> {
        self.header
            .append(header::COOKIE, HeaderValue::from_str(cookie)?);
        Ok(())
    }
    // todo: make this method async
    pub fn load_netscape_cookie(&mut self, cookie: impl AsRef<Path>) -> Result<(), ClientError> {
        let cookies = nescookie::open(cookie)?
            .iter()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .fold(String::new(), |acc, x| acc + &x + ";");
        self.push_cookie(&cookies)?;
        Ok(())
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
    pub fn client_mut(&mut self) -> &mut reqwest::Client {
        &mut self.inner
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

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
