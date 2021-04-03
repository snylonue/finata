use crate::utils::Client;
use crate::Config;
use crate::{error as err, Origin};
use crate::{utils, Track};
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
/// ?bvid={} or ?aid={}
pub const VIDEO_INFO_API: &str = "https://api.bilibili.com/x/web-interface/view";

#[derive(Debug)]
pub enum Id {
    Av(String),
    Bv(String),
}

pub struct Video {
    client: Client,
    id: Id,
    page: Option<usize>,
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

impl Video {
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
        let page = url
            .query_pairs()
            .find_map(|(key, v)| if key == "p" { v.parse().ok() } else { None });
        Ok(Self::with_id(id, page))
    }
    pub fn with_id(id: String, page: Option<usize>) -> Self {
        Self::with_client(Client::with_header(HEADERS.clone()), id, page)
    }
    pub fn with_client(client: Client, id: String, page: Option<usize>) -> Self {
        Self {
            client,
            id: Id::new(&id),
            page,
        }
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
    pub async fn video_info_json(&self) -> Result<Value, Error> {
        let url = match &self.id {
            Id::Av(av) => format!("{}?aid={}", VIDEO_INFO_API, av),
            Id::Bv(bv) => format!("{}?bvid={}", VIDEO_API, bv),
        }
        .parse()?;
        self.client.send_json_request(url).await
    }
    pub async fn title(&self) -> Result<String, Error> {
        let info = self.video_info_json().await?;
        match info["data"]["title"] {
            Value::String(ref s) => Ok(s.to_owned()),
            _ => err::InvalidResponse { resp: info }.fail(),
        }
    }
    async fn current_page(&self) -> Result<Value, Error> {
        let playlist = self.playlist_json().await?;
        match playlist.get(self.page.unwrap_or(1) - 1) {
            Some(res) => Ok(res.clone()),
            _ => err::InvalidResponse { resp: playlist }.fail(),
        }
    }
}

#[async_trait::async_trait]
impl Extract for Video {
    async fn extract(&mut self) -> crate::FinaResult {
        let page = self.current_page().await?;
        let cid = extract_cid(&page)?;
        let title = self.title().await.unwrap_or_default();
        let mut tracks = Vec::new();
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
            (Some(vurl), Some(aurl)) => tracks
                .extend_from_slice(&[Track::Video(vurl.parse()?), Track::Audio(aurl.parse()?)]),
            (Some(url), _) => tracks.push(Track::Video(url.parse()?)),
            (_, Some(url)) => tracks.push(Track::Audio(url.parse()?)),
            _ => return err::InvalidResponse { resp: info }.fail(),
        };
        let origin = Origin::new(tracks, page["part"].as_str().unwrap_or_default().to_owned());
        Ok(Finata::new(vec![origin], title))
    }
}

impl Config for Video {
    fn client(&self) -> &Client {
        &self.client
    }
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

fn extract_cid(data: &Value) -> Result<u64, Error> {
    data["cid"]
        .as_u64()
        .ok_or(err::InvalidResponse { resp: data.clone() }.build())
}
