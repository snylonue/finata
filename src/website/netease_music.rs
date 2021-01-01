use crate::{error as err, value_to_string};
use crate::{utils, Error, Extract, Finata, Origin};
use lazy_static::lazy_static;
use reqwest::header;
use serde_json::Value;
use sugars::hmap;
use url::Url;
use utils::Client;

const SONG_URL_API: &'static str = "https://music.163.com/api/song/enhance/player/url";
#[allow(dead_code)]
const SONG_DETIAL_API: &'static str = "https://music.163.com/api/song/detail";
#[allow(dead_code)]
const PLAYLIST_DETAIL_API: &str = "https://music.163.com/api/v3/playlist/detail";

lazy_static! {
    static ref HEADERS: header::HeaderMap = crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://music.163.com",
    };
}

#[derive(Debug, Clone)]
pub struct Song {
    id: u64,
    client: Client,
}

impl Song {
    pub fn from_id(id: u64) -> Self {
        Self {
            id,
            client: Client::with_header(HEADERS.clone()),
        }
    }
    pub async fn raw_url(&self) -> Result<Url, Error> {
        let url_info: Value = self
            .client
            .client()
            .post(SONG_URL_API)
            .headers(HEADERS.clone())
            .form(&hmap! {
                "ids" => format!("[{}]", self.id),
                "br" => String::from("999000")
            })
            .send()
            .await?
            .json()
            .await?;
        match url_info["data"][0]["url"] {
            Value::String(ref url) => Ok(Url::parse(url)?),
            _ => err::InvalidResponse { resp: url_info }.fail(),
        }
    }
    pub async fn title(&self) -> Result<String, Error> {
        let details: Value = self
            .client
            .client()
            .post(SONG_DETIAL_API)
            .headers(HEADERS.clone())
            .form(&hmap! { "ids" => format!("[{}]", self.id) })
            .send()
            .await?
            .json()
            .await?;
        let error = || {
            err::InvalidResponse {
                resp: details.clone(),
            }
            .build()
        };
        let name = value_to_string!(details["songs"][0]["name"]).ok_or_else(error)?;
        let arthor = details["songs"][0]["artists"]
            .as_array()
            .ok_or_else(error)?
            .iter()
            .filter_map(|s| s.as_str())
            .collect::<Vec<_>>();
        match arthor.len() {
            0 => Ok(name),
            _ => Ok(format!("{} - {}", arthor.join(","), name)),
        }
    }
}
#[derive(Debug, Clone)]
pub struct List {
    id: u64,
    client: Client,
}

#[async_trait::async_trait]
impl Extract for Song {
    async fn extract(&mut self) -> Result<crate::Finata, Error> {
        let url = self.raw_url().await?;
        let title = self.title().await?; // todo: implement title parse
        Ok(Finata::new(
            vec![Origin::new(crate::Format::Audio, url)],
            title,
        ))
    }
}
