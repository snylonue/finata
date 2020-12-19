use crate::Extract;
use crate::{error as err, utils, Format};
use crate::{Error, Finata};
use lazy_static::lazy_static;
use reqwest::{header, Client};
use serde_json::Value;
use snafu::ResultExt;
use std::iter::once;
use url::Url;

lazy_static! {
    static ref HEADERS: header::HeaderMap = crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://www.pixiv.net/",
    };
    static ref IMAGE_API: Url = Url::parse("https://www.pixiv.net/ajax/illust/").unwrap();
}

#[derive(Debug)]
pub struct Pixiv {
    client: Client,
    pid: String,
}

impl Pixiv {
    pub fn new(s: &str) -> Result<Self, Error> {
        let url: Url = Url::parse(s)?;
        let pid = url
            .path_segments()
            .ok_or(Error::InvalidUrl {
                url: url.to_owned(),
            })?
            .next_back()
            .ok_or(Error::InvalidUrl {
                url: url.to_owned(),
            })?
            .to_owned();
        Ok(Self::with_pid(pid))
    }
    pub fn with_pid(pid: String) -> Self {
        Self::with_client(utils::CLIENT.clone(), pid)
    }
    pub fn with_client(client: Client, pid: String) -> Self {
        Self { client, pid }
    }
    async fn raw_url_json(&self) -> Result<Value, Error> {
        let url = IMAGE_API.join(&format!("{}/pages", self.pid)).unwrap();
        let data = self
            .client
            .get(url.clone())
            .send()
            .await
            .context(err::NetworkError { url: url.clone() })?
            .json()
            .await?;
        Ok(data)
    }
    async fn meta_json(&self) -> Result<Value, Error> {
        let url = IMAGE_API.join(&self.pid).unwrap();
        let data = self
            .client
            .get(url.clone())
            .send()
            .await
            .context(err::NetworkError { url: url.clone() })?
            .json()
            .await?;
        Ok(data)
    }
    async fn raw_urls(&self) -> Result<Vec<Value>, Error> {
        let mut data = self.raw_url_json().await?;
        match data["body"] {
            Value::Array(ref mut urls) => Ok(urls.to_owned()),
            _ => err::InvalidResponse { resp: data }.fail(),
        }
    }
}

#[async_trait::async_trait]
impl Extract for Pixiv {
    async fn extract(&mut self) -> Box<dyn Iterator<Item = Result<crate::Finata, crate::Error>>> {
        let title = match self.meta_json().await {
            Ok(resp) => resp["body"]["title"].as_str().unwrap_or("").to_owned(),
            Err(e) => return Box::new(once(Err(e))),
        };
        let urls = match self.raw_urls().await {
            Ok(urls) => urls,
            Err(e) => return Box::new(once(Err(e))),
        };
        let it = urls.into_iter().map(move |v| {
            let url_data = &v["urls"]["original"];
            match url_data {
                Value::String(ref url) => Url::parse(url).map_err(Error::from).map(|raw| Finata {
                    raw,
                    title: title.clone(),
                    format: Format::Image,
                }),
                _ => err::InvalidResponse { resp: v }.fail(),
            }
        });
        Box::new(it)
    }
}
