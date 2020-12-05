use crate::error::Error;
use crate::utils;
use crate::utils::CLIENT;
use crate::value_to_string;
use crate::FinataData;
use crate::Format;
use lazy_static::lazy_static;
use reqwest::header;
use reqwest::Url;
use serde_json::Value;

lazy_static! {
    static ref HEADERS: header::HeaderMap = crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://www.pixiv.net/",
    };
    static ref IMAGE_API: Url = Url::parse("https://www.pixiv.net/ajax/illust/").unwrap();
}

pub struct Pixiv {
    url: Url,
}

impl Pixiv {
    pub const fn new(url: Url) -> Self {
        Self { url }
    }
    fn pid(&self) -> Result<&str, Error> {
        self.url
            .path_segments()
            .map(|sp| sp.last())
            .flatten()
            .ok_or(Error::invalid_input_url(&self.url))
    }
    pub async fn extract(self) -> Result<FinataData, Error> {
        let url = self.url.clone();
        let data = self.meta_json().await?;
        let raw_url = data["body"]["urls"]["original"]
            .as_str()
            .ok_or(Error::None)?;
        let title = value_to_string!(data["body"]["title"]);
        Ok(FinataData::new(
            url,
            vec![(raw_url.parse()?, Format::Image)],
            HEADERS.clone(),
            title,
        ))
    }
    pub async fn meta(self) -> Result<String, Error> {
        let pid = self.pid()?;
        let data = CLIENT
            .get(IMAGE_API.join(pid)?)
            .send()
            .await?
            .text()
            .await?;
        Ok(data)
    }
    pub fn meta_url(&self) -> Result<FinataData, Error> {
        let pid = self.pid()?;
        Ok(FinataData::new(
            self.url.clone(),
            vec![(IMAGE_API.join(pid)?, Format::Text)],
            HEADERS.clone(),
            None,
        ))
    }
    pub async fn meta_json(self) -> Result<Value, Error> {
        let pid = self.pid()?;
        let data = CLIENT
            .get(IMAGE_API.join(pid)?)
            .send()
            .await?
            .json()
            .await?;
        Ok(data)
    }
}
