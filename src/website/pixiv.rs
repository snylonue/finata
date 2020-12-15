use reqwest::{Client, header};
use crate::{Error, Finata, FinataError};
use url::Url;
use std::convert::TryInto;
use crate::NetWorkError;
use snafu::ResultExt;
use snafu::OptionExt;
use crate::utils;
use crate::Extract;
// use crate::
use serde_json::Value;
use lazy_static::lazy_static;

lazy_static! {
    static ref HEADERS: header::HeaderMap = crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://www.pixiv.net/",
    };
    static ref IMAGE_API: Url = Url::parse("https://www.pixiv.net/ajax/illust/").unwrap();
}

pub struct Pixiv {
    client: Client,
    pid: String,
}

impl Pixiv {
    pub fn new(s: impl TryInto<Url, Error=url::ParseError>) -> Result<Self, Error> {
        let url: Url = s.try_into()?;
        let pid = url.path_segments().ok_or(Error::InvalidUrl { url: url.to_owned() })?
            .next_back()
            .ok_or(Error::InvalidUrl { url: url.to_owned() })?
            .to_owned();
        Ok(Self::with_pid(pid))
    }
    pub fn with_pid(pid: String) -> Self {
        Self::with_client(utils::CLIENT.clone(), pid)
    }
    pub fn with_client(client: Client, pid: String) -> Self {
        Self { client, pid }
    }
    async fn meta_json(&self) -> Result<Value, Error> {
        let url = IMAGE_API.join(&format!("{}/pages", self.pid)).unwrap();
        dbg!(&url);
        self
            .client
            .get(url.clone())
            .send()
            .await
            .context(NetWorkError { url: url.clone() })?
            .json()
            .await
            .context(NetWorkError { url: url.clone() })
    }
}

use std::iter::once;

#[async_trait::async_trait]
impl Extract for Pixiv {
    async fn extract(&mut self) -> Box<dyn Iterator<Item=Result<crate::Finata, crate::FinataError>>> {
        let data = self.meta_json().await.unwrap();
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
        match data["body"].as_array() {
            None => return Box::new(once(crate::InValidResponse { resp: data.clone() }.fail().map_err(Into::into))),
            Some(arr) => {
               let it = arr.clone().into_iter().map(move |v| {
                    let url = v["urls"]["original"].as_str();
                    match url.map(|u| Url::parse(u)) {
                        Some(url) => url.map_err(Error::from).map_err(FinataError::from).map(|raw| Finata { raw, title: "".to_string(), format: crate::Format::Image }),
                        None => crate::InValidResponse { resp: data.clone() }.fail().map_err(Into::into)
                    }
                });
                return Box::new(it);
            }
        };
        todo!()
    }
}