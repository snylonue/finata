use reqwest::Url;
use reqwest::header;
use lazy_static::lazy_static;
use serde_json::Value;
use crate::FinataData;
use crate::Format;
use crate::error::Error;
use crate::value_to_string;
use crate::utils;
use crate::utils::CLIENT;

lazy_static! {
    static ref HEADERS: header::HeaderMap = crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://www.pixiv.net/",
    };
    static ref IMAGE_API: Url = Url::parse("https://www.pixiv.net/ajax/illust/").unwrap();
}

pub struct Pixiv {
    url: Url
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
        let pid = self.pid()?;
        let data: Value = CLIENT.get(IMAGE_API.join(pid)?)
            .send().await?
            .json().await?;
        let url = value_to_string!(data["body"]["urls"]["original"]).ok_or(Error::None)?;
        let title = value_to_string!(data["body"]["title"]);
        Ok(FinataData::new(self.url, vec![(url.parse()?, Format::Image)], HEADERS.clone(), title))
    }
}