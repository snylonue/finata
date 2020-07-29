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

const SONG_URL_API: &'static str = "https://music.163.com/api/song/enhance/player/url";
const SONG_DETIAL_API: &'static str = "https://music.163.com/api/song/detail";
const PLAYLIST_DETAIL_API: &str = "https://music.163.com/api/v3/playlist/detail";

lazy_static! {
    static ref HEADERS: header::HeaderMap = crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://music.163.com",
    };
}

pub struct Song {
    url: Url,
}
pub struct List {
    url: Url,
}

impl Song {
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
        let url_info: Value = CLIENT.post(SONG_URL_API)
            .headers(HEADERS.clone())
            .form(&hmap! {
                "ids" => format!("[{}]", self.id()?),
                "br" => String::from("999000")
            })
            .send().await?
            .json().await?;
        match url_info["data"][0]["url"] {
            Value::String(ref url) => Ok(Url::parse(url).unwrap()),
            _ => if url_info["data"][0]["code"] == -110 {
                Err(Error::needs_vip())
            } else {
                Err(Error::None)
            }
        }
    }
    pub async fn title(&self) -> Result<String, Error> {
        let details: Value = CLIENT.post(SONG_DETIAL_API)
            .headers(HEADERS.clone())
            .form(&hmap!{ "ids" => format!("[{}]", self.id()?) })
            .send().await?
            .json().await?;
        let name = value_to_string!(details["songs"][0]["name"]).ok_or(Error::None)?;
        let arthor = details["songs"][0]["artists"]
            .as_array().ok_or(Error::None)?
            .iter()
            .filter_map(|s| s.as_str())
            .collect::<Vec<_>>();
        match arthor.len() {
            0 => Ok(name),
            _ => Ok(format!("{} - {}", arthor.join(","), name)),
        }
    }
    pub async fn extract(self) -> Result<FinataData, Error> {
        let url = self.raw_url().await?;
        let title = self.title().await.ok();
        Ok(FinataData::new(self.url, vec![(url, Format::Audio)], HEADERS.clone(), title))
    }
}
impl List {
    pub const fn new(url: Url) -> Self {
        Self { url }
    }
    fn id(&self) -> Result<&str, Error> {
        self.url
            .fragment()
            .map(|s| s.trim_start_matches("/playlist?id=").trim_end_matches('/'))
            .ok_or(Error::invalid_input_url(&self.url))
    }
    pub async fn extract(self) -> Result<FinataData, Error> {
        let id = self.id()?;
        let url_info: Value = CLIENT.post(PLAYLIST_DETAIL_API)
            .query(&[("id", id)])
            .headers(HEADERS.clone())
            .send().await?
            .json().await?;
        let track_ids = url_info["playlist"]["trackIds"].as_array().ok_or(Error::None)?;
        let mut songs = Vec::with_capacity(track_ids.len());
        for track_id in track_ids.iter().filter_map(|v| v["id"].as_u64()) { //skip invalid value
            let song_url = format!("https://music.163.com/#/song?id={}", track_id);
            let raw_url = match Song::new(Url::parse(&song_url).unwrap()).raw_url().await {
                Ok(r) => r,
                Err(Error::PermissionDenied(_)) => continue, // skip if a vip song is met
                Err(e) => return Err(e)
            };
            songs.push((raw_url, Format::Audio));
        }
        let name = value_to_string!(url_info["playlist"]["name"]);
        Ok(FinataData::new(self.url, songs, HEADERS.clone(), name))
    }
}