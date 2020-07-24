use reqwest::Url;
use reqwest::header;
use lazy_static::lazy_static;
use serde_json::Value;
use sugars::hmap;
use crate::FinataData;
use crate::Format;
use crate::error::Error;
use crate::value_to_string;
use crate::utils;
use crate::utils::CLIENT;

lazy_static! {
    static ref HEADERS: header::HeaderMap = {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, utils::UA.clone());
        headers.insert(header::REFERER, header::HeaderValue::from_static("https://music.163.com"));
        headers
    };
}

pub struct NeteaseCloudMusic {
    url: Url,
}

impl NeteaseCloudMusic {
    const SONG_URL_API: &'static str = "https://music.163.com/api/song/enhance/player/url";
    const SONG_DETIAL_API: &'static str = "https://music.163.com/api/song/detail";
    pub const fn new(url: Url) -> Self {
        Self { url }
    }
    fn id(&self) -> Result<&str, Error> {
        self.url
            .fragment()
            .map(|s| s.trim_start_matches("/song?id=").trim_end_matches('/'))
            .ok_or(Error::invalid_input_url(&self.url))
    }
    pub async fn raw_url(&self) -> Result<Url, Error> {
        let form = hmap! {
            "ids" => format!("[{}]", self.id()?),
            "br" => String::from("999000")
        };
        let url_info: Value = CLIENT.post(Self::SONG_URL_API)
            .headers(HEADERS.clone())
            .form(&form)
            .send().await?
            .json().await?;
        let url = value_to_string!(url_info["data"][0]["url"]).ok_or(Error::None)?;
        Ok(Url::parse(&url).unwrap())
    }
    pub async fn title(&self) -> Result<String, Error> {
        let form = hmap! {
            "ids" => format!("[{}]", self.id()?)
        };
        let details: Value = CLIENT.post(Self::SONG_DETIAL_API)
            .headers(HEADERS.clone())
            .form(&form)
            .send().await?
            .json().await?;
        let name = value_to_string!(details["songs"][0]["name"]).ok_or(Error::None)?;
        let arthor = details["songs"][0]["artists"]
            .as_array().ok_or(Error::None)?
            .iter()
            .filter_map(|s| value_to_string!(s))
            .collect::<Vec<_>>();
        match arthor.len() {
            0 => Ok(name),
            _ => Ok(format!("{} - {}", arthor.join(","), name)),
        }
    }
    pub async fn extract(self) -> Result<FinataData, Error> {
        let url = self.raw_url().await?;
        let title = self.title().await?;
        Ok(FinataData::new(self.url, vec![(url, Format::Audio)], HEADERS.clone(), Some(title)))
    }
}