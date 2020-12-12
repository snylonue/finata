use crate::utils;
use crate::utils::CLIENT;
use crate::value_to_string;
use crate::Extract;
use crate::Finata;
use crate::Format;
use lazy_static::lazy_static;
use reqwest::header;
use reqwest::Client;
use reqwest::Url;
use serde_json::Value;
use snafu::OptionExt;
use snafu::ResultExt;
use snafu::Snafu;
use std::iter::once;

lazy_static! {
    static ref HEADERS: header::HeaderMap = crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://www.pixiv.net/",
    };
    static ref IMAGE_API: Url = Url::parse("https://www.pixiv.net/ajax/illust/").unwrap();
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("network error: {}", source))]
    NetWorkError {
        source: reqwest::Error,
    },
    #[snafu(display("invalid url `{}`", url))]
    InvalidUrl {
        url: Url,
    },
    ParseUrlError {
        source: url::ParseError,
    },
    /// failed to extract infomation needed
    InvalidResponse,
}

pub struct Pixiv {
    url: Url,
    client: Client,
    data: Option<Value>,
}

impl Pixiv {
    pub fn new(url: Url) -> Self {
        Self::with_client(url, CLIENT.clone())
    }
    pub fn with_client(url: Url, client: Client) -> Self {
        Self {
            url,
            client,
            data: None,
        }
    }
    fn pid(&self) -> Result<&str, Error> {
        let pid = self.url.path_segments().map(|sp| sp.last()).flatten();
        snafu::ensure!(
            pid.is_some(),
            InvalidUrl {
                url: self.url.clone()
            }
        );
        Ok(pid.unwrap())
    }
    pub async fn meta_json(&mut self) -> Result<&Value, Error> {
        match self.data {
            Some(ref v) => Ok(v),
            None => {
                let pid = self.pid()?;
                let data = self
                    .client
                    .get(IMAGE_API.join(pid).unwrap())
                    .send()
                    .await
                    .context(NetWorkError)?
                    .json()
                    .await
                    .context(NetWorkError)?;
                self.data.replace(data);
                Ok(self.data.as_ref().unwrap())
            }
        }
    }
    async fn extract(&mut self) -> Result<Finata, Error> {
        let url = self.url.clone();
        let data = self.meta_json().await?;
        let raw_url = data["body"]["urls"]["original"]
            .as_str()
            .context(InvalidResponse)?;
        let title = value_to_string!(data["body"]["title"]);
        Ok(Finata::new(
            url,
            raw_url.parse().context(ParseUrlError)?,
            Format::Image,
            title,
        ))
    }
    pub fn meta_url(&self) -> Result<Url, Error> {
        let pid = self.pid()?;
        Ok(IMAGE_API.join(pid).unwrap())
    }
}
#[async_trait::async_trait]
impl Extract for Pixiv {
    async fn extract(
        &mut self,
    ) -> Box<dyn Iterator<Item = Result<crate::Finata, crate::error::Error>>> {
        Box::new(once(self.extract().await.map_err(Into::into)))
    }

    fn header(&self) -> header::HeaderMap {
        HEADERS.clone()
    }
}
