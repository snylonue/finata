use crate::{error as err, utils, AsClient, Error, Extract, FinaResult, Origin, Playlist};
use once_cell::sync::Lazy;
use reqwest::header::{self, HeaderMap};
use serde_json::Value;
use sugars::hmap;
use url::Url;
use utils::Client;

const SONG_URL_API: &str = "https://music.163.com/api/song/enhance/player/url";
const SONG_DETIAL_API: &str = "https://music.163.com/api/song/detail";
const PLAYLIST_DETAIL_API: &str = "https://music.163.com/api/v3/playlist/detail";

static HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://music.163.com",
    }
});

#[derive(Debug, Clone)]
pub struct Song {
    id: u64,
    client: Client,
}

// todo: refactor
impl Song {
    pub fn new(url: &str) -> Result<Self, Error> {
        let url: Url = url.parse()?;
        url.fragment()
            .map(|s| s.trim_start_matches("/song?id=").trim_end_matches('/'))
            .ok_or_else(|| Error::InvalidUrl { url: url.clone() })?
            .parse()
            .map(Self::with_id)
            .map_err(|_| Error::InvalidUrl { url })
    }
    pub fn with_id(id: u64) -> Self {
        Self::with_client(Client::with_header(HEADERS.clone()), id)
    }
    pub fn with_client(client: Client, id: u64) -> Self {
        Self { id, client }
    }
    pub fn client(&self) -> &Client {
        &self.client
    }
    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
    pub async fn raw_url(&self) -> Result<Url, Error> {
        let url_info: Value = self
            .client
            .post_form_json(
                Url::parse(SONG_URL_API).unwrap(),
                &hmap! {
                    "ids" => format!("[{}]", self.id),
                    "br" => String::from("999000")
                },
            )
            .await?;
        match url_info["data"][0]["url"] {
            Value::String(ref url) => Ok(Url::parse(url)?),
            _ => err::InvalidResponse { resp: url_info }.fail(),
        }
    }
    pub async fn title(&self) -> Result<String, Error> {
        let details: Value = self
            .client
            .post_form_json(
                Url::parse(SONG_DETIAL_API).unwrap(),
                &hmap! { "ids" => format!("[{}]", self.id) },
            )
            .await?;
        let error = || {
            err::InvalidResponse {
                resp: details.clone(),
            }
            .build()
        };
        let name = details["songs"][0]["name"].as_str().ok_or_else(error)?;
        let arthor = details["songs"][0]["artists"]
            .as_array()
            .ok_or_else(error)?
            .iter()
            .filter_map(|s| s.as_str())
            .collect::<Vec<_>>();
        match arthor.len() {
            0 => Ok(name.to_owned()),
            _ => Ok(format!("{} - {}", arthor.join(","), name)),
        }
    }
}
#[derive(Debug, Clone)]
pub struct PlayList {
    id: u64,
    client: Client,
    ignore_vip: bool,
}

impl PlayList {
    pub fn new(url: &str) -> Result<Self, Error> {
        let url: Url = url.parse()?;
        url.fragment()
            .map(|s| s.trim_start_matches("/playlist?id=").trim_end_matches('/'))
            .ok_or_else(|| Error::InvalidUrl { url: url.clone() })?
            .parse()
            .map(Self::with_id)
            .map_err(|_| Error::InvalidUrl { url })
    }
    pub fn with_id(id: u64) -> Self {
        Self::extracts_vip(id, false)
    }
    pub fn with_client(client: Client, id: u64, vip: bool) -> Self {
        Self {
            id,
            client,
            ignore_vip: !vip,
        }
    }
    pub fn extracts_vip(id: u64, vip: bool) -> Self {
        Self::with_client(Client::with_header(HEADERS.clone()), id, vip)
    }
}

#[async_trait::async_trait]
impl Extract for Song {
    async fn extract(&mut self) -> FinaResult {
        let url = self.raw_url().await?;
        let title = self.title().await?;
        Ok(Playlist::new(
            vec![Origin::audio(url, String::new())],
            title,
        ))
    }
}

impl AsClient for Song {
    fn client(&self) -> &Client {
        &self.client
    }
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

#[async_trait::async_trait]
impl Extract for PlayList {
    async fn extract(&mut self) -> FinaResult {
        let url_info: Value = self
            .client
            .post_json(
                Url::parse_with_params(PLAYLIST_DETAIL_API, &[("id", self.id.to_string())])
                    .unwrap(),
            )
            .await?;
        let error = || {
            err::InvalidResponse {
                resp: url_info.clone(),
            }
            .build()
        };
        let track_ids = url_info["playlist"]["trackIds"]
            .as_array()
            .ok_or_else(error)?;
        let s = futures_util::future::join_all(
            track_ids
                .iter()
                .filter_map(|v| v["id"].as_u64()) // skip invalid id
                .map(|id| async move { Song::with_id(id).raw_url().await }),
        )
        .await;
        let mut songs = Vec::with_capacity(track_ids.len());
        for resp in s {
            match resp {
                Ok(raw_url) => songs.push(Origin::audio(raw_url, String::new())),
                Err(e) => match e {
                    // ignore vip songs
                    Error::InvalidResponse { ref resp }
                        if self.ignore_vip && resp["data"][0]["code"] == -110 => {}
                    _ => return Err(e),
                },
            };
        }
        let name = url_info["playlist"]["name"]
            .as_str()
            .unwrap_or("")
            .to_owned();
        Ok(Playlist::new(songs, name))
    }
}

impl AsClient for PlayList {
    fn client(&self) -> &Client {
        &self.client
    }
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}
