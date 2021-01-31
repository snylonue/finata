use crate::utils::Client;
use crate::{error as err, Origin};
use crate::{utils, Format};
use crate::{Error, Extract, Finata};
use lazy_static::lazy_static;
use reqwest::header;
use serde_json::Value;
use url::Url;

lazy_static! {
    static ref HEADERS: header::HeaderMap = crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://www.bilibili.com",
    };
}

/// add ?bvid={} or ?aid={}
const CID_API: &str = "https://api.bilibili.com/x/player/pagelist";
/// ?cid={}&qn={}&avid={} or ?cid={}&qn={}&bvid={}
const VIDEO_API: &str = "https://api.bilibili.com/x/player/playurl";

#[derive(Debug)]
pub enum Id {
    Av(String),
    Bv(String),
}

pub struct Bilibili {
    client: Client,
    id: Id,
}

impl Id {
    pub fn new(id: &str) -> Self {
        if id.starts_with("av") {
            Self::Av(id.trim_start_matches("av").to_owned())
        } else if id.starts_with("BV") {
            Self::Bv(id.to_owned())
        } else {
            todo!()
        }
    }
}

impl Bilibili {
    pub fn new(s: &str) -> Result<Self, Error> {
        let url: Url = Url::parse(s)?;
        let id = url
            .path_segments()
            .map(|mut it| it.next_back())
            .flatten()
            .ok_or(Error::InvalidUrl {
                url: url.to_owned(),
            })?
            .to_owned();
        Ok(Self::with_id(id))
    }
    pub fn with_id(id: String) -> Self {
        Self::with_client(Client::with_header(HEADERS.clone()), id)
    }
    pub fn with_client(client: Client, id: String) -> Self {
        Self {
            client,
            id: Id::new(&id),
        }
    }
    pub fn client(&self) -> &Client {
        &self.client
    }
    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
    pub async fn playlist_json(&self) -> Result<Vec<Value>, Error> {
        let url = match &self.id {
            Id::Av(av) => format!("{}?aid={}", CID_API, av),
            Id::Bv(bv) => format!("{}?bvid={}", CID_API, bv),
        }
        .parse()?;
        let data = self.client.send_json_request(url).await?;
        match data["data"] {
            Value::Array(ref cids) => Ok(cids.clone()),
            _ => err::InvalidResponse { resp: data }.fail(),
        }
    }
    /// Returns dash url
    pub async fn video_url_json(&self, cid: u64) -> Result<Value, Error> {
        let url = {
            let mut tmp = format!("{}?cid={}&fnval=16&fourk=1&", VIDEO_API, cid);
            match self.id {
                Id::Av(ref avid) => tmp.push_str(&format!("avid={}", avid)),
                Id::Bv(ref bvid) => tmp.push_str(&format!("bvid={}", bvid)),
            };
            tmp.parse()?
        };
        self.client.send_json_request(url).await
    }
    pub async fn video_dash_urls(&self, cid: u64) -> Result<Value, Error> {
        let mut data = self.video_url_json(cid).await?;
        match &mut data["data"]["dash"] {
            Value::Null => err::InvalidResponse { resp: data }.fail(),
            res @ _ => Ok(res.take()),
        }
    }
}

#[async_trait::async_trait]
impl Extract for Bilibili {
    async fn extract(&mut self) -> crate::FinaResult {
        let cids = self.playlist_json().await?;
        let cids = cids
            .into_iter()
            .map(extract_cid)
            .collect::<Result<Vec<_>, Error>>()?;
        let mut urls = Vec::with_capacity(cids.len());
        for cid in cids {
            let info = self.video_dash_urls(cid).await?;
            let video_url = info["video"]
                .as_array()
                .map(|data| data.iter().find_map(|data| data["baseUrl"].as_str()))
                .flatten();
            let audio_url = info["audio"]
                .as_array()
                .map(|data| data.iter().find_map(|data| data["baseUrl"].as_str()))
                .flatten();
            match (video_url, audio_url) {
                (Some(vurl), Some(aurl)) => urls.extend_from_slice(&[
                    Origin::new(Format::Video, Url::parse(vurl)?),
                    Origin::new(Format::Audio, Url::parse(aurl)?),
                ]),
                (Some(url), _) => urls.push(Origin::new(Format::Video, Url::parse(url)?)),
                (_, Some(url)) => urls.push(Origin::new(Format::Audio, Url::parse(url)?)),
                _ => return err::InvalidResponse { resp: info }.fail(),
            };
        }
        Ok(Finata::new(urls, String::new()))
    }
}

fn extract_cid(data: Value) -> Result<u64, Error> {
    data["cid"]
        .as_u64()
        .ok_or(err::InvalidResponse { resp: data }.build())
}
