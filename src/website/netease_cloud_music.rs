use reqwest::Url;
use reqwest::header;
use lazy_static::lazy_static;
use serde_json::Value;
use crate::FinataData;
use crate::Format;

macro_rules! hashmap {
    ($($name: expr => $content: expr),*) => {
        {
            let mut hm = ::std::collections::HashMap::new();
            $(hm.insert($name, $content);)*
            hm
        }
    };
}
macro_rules! value_to_string {
    ($v: expr) => {
        match $v {
            serde_json::Value::String(ref s) => Some(s.clone()),
            _ => None,
        }
    };
    ($v: expr, $or: expr) => {
        match $v {
            serde_json::Value::String(ref s) => Some(s.clone()),
            _ => $crate::value_to_string!($or),
        }
    };
}

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
    static ref HEADERS: header::HeaderMap = {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36".parse().unwrap());
        headers.insert(header::REFERER, "https://music.163.com".parse().unwrap());
        headers
    };
}

pub struct NeteaseCloudMusic {
    url: Url,
}

impl<'a> NeteaseCloudMusic {
    const SONG_URL_API: &'static str = "https://music.163.com/api/song/enhance/player/url";
    const _SONG_DETIAL_API: &'static str = "https://music.163.com/api/song/detail";
    pub const fn new(url: Url) -> Self {
        Self { url }
    }
    pub async fn extract(self) -> Result<FinataData, reqwest::Error> {
        let id = self.url.fragment()
            .map(|s| s.trim_start_matches("/song?id=").trim_end_matches('/'))
            .unwrap();
        let form = hashmap!{
            "ids" => format!("[{}]", dbg!(id)),
            "br" => String::from("999000")
        };
        let url_info: Value = CLIENT.post(Self::SONG_URL_API)
            .headers(HEADERS.clone())
            .form(&form)
            .send().await?
            .json().await?;
        let url = value_to_string!(url_info["data"][0]["url"]).unwrap().to_string();
        let _ext = value_to_string!(url_info["data"][0]["type"]).unwrap();
        Ok(FinataData::new(self.url, vec![(Url::parse(&url).unwrap(), Format::Audio)], HEADERS.clone(), None))
    }
}