use crate::Extract;
use crate::NetworkError;
use crate::InvalidResponse;
use crate::{utils, Format};
use crate::{Error, Finata};
use lazy_static::lazy_static;
use reqwest::{header, Client};
use serde_json::Value;
use snafu::{ResultExt};
use std::convert::TryInto;
use url::Url;
use std::iter::once;


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

#[derive(Debug)]
pub struct Iter {
    pub(crate) data: std::vec::IntoIter<Value>,
}

impl Pixiv {
    pub fn new(s: impl TryInto<Url, Error = url::ParseError>) -> Result<Self, Error> {
        let url: Url = s.try_into()?;
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
    async fn meta_json(&self) -> Result<Value, Error> {
        let url = IMAGE_API.join(&format!("{}/pages", self.pid)).unwrap();
        self.client
            .get(url.clone())
            .send()
            .await
            .context(NetworkError { url: url.clone() })?
            .json()
            .await
            .context(NetworkError { url: url.clone() })
    }
}


#[async_trait::async_trait]
impl Extract for Pixiv {
    async fn extract(
        &mut self,
    ) -> Box<dyn Iterator<Item = Result<crate::Finata, crate::Error>>> {
        let data = self.meta_json().await.unwrap();
        match data["body"].as_array() {
            None => Box::new(once(
                InvalidResponse { resp: data.clone() }
                    .fail()
            )),
            Some(arr) => {
                let it = arr.clone().into_iter().map(move |v| {
                    let url = v["urls"]["original"].as_str();
                    match url.map(|u| Url::parse(u)) {
                        Some(url) => {
                            url.map_err(Error::from)
                                .map(|raw| Finata {
                                    raw,
                                    title: "".to_string(),
                                    format: Format::Image,
                                })
                        }
                        None => InvalidResponse { resp: data.clone() }
                            .fail()
                            .map_err(Into::into),
                    }
                });
                Box::new(it)
            }
        }
    }
}
