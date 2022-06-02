use crate::utils::Client;
use crate::AsClient;
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
/// add ?season_id={} or ?ep_id={}
const BANGUMI_CID_API: &str = "https://api.bilibili.com/pgc/view/web/season";
/// ?cid={}&qn={}&avid={} or ?cid={}&qn={}&bvid={}
const VIDEO_API: &str = "https://api.bilibili.com/x/player/playurl";
/// ?bvid={} or ?aid={}
pub const VIDEO_INFO_API: &str = "https://api.bilibili.com/x/web-interface/view";
/// ?cid={}&quality={}&platform={}
pub const LIVE_API: &str = "https://api.live.bilibili.com/room/v1/Room/playUrl";
/// ?id={}
pub const LIVE_INFO_API: &str = "http://api.live.bilibili.com/room/v1/Room/room_init";
/// ?mid={}
pub const SPACE_INFO_API: &str = "https://api.bilibili.com/x/space/acc/info";

#[derive(Debug)]
pub enum Id {
    Av(u64),
    Bv(String),
    Ep(u64),
    Ss(u64),
}

pub struct BaseExtractor {
    aid: u64,
    cid: u64,
    client: Client,
}

pub struct Video {
    client: Client,
    id: Id,
    page: Option<usize>,
}

pub struct Bangumi {
    client: Client,
    id: Id,
}

pub struct BaseLiveExtractor {
    cid: u64,
    client: Client,
}

pub struct Live {
    id: u64,
    client: Client,
}

impl Id {
    pub fn from_url(url: &Url) -> Result<Self, Error> {
        url.path_segments()
            .and_then(|it| it.filter(|p| !p.is_empty()).next_back())
            .and_then(Self::new)
            .ok_or_else(|| Error::InvalidUrl {
                url: url.to_owned(),
            })
    }
    pub fn new(id: &str) -> Option<Self> {
        if id.starts_with("av") {
            id.trim_start_matches("av").parse().ok().map(Self::Av)
        } else if id.starts_with("BV") {
            Some(Self::Bv(id.to_owned()))
        } else if id.starts_with("ep") {
            id.trim_start_matches("ep").parse().ok().map(Self::Ep)
        } else if id.starts_with("ss") {
            id.trim_start_matches("ss").parse().ok().map(Self::Ss)
        } else {
            None
        }
    }
    fn as_cid_api(&self) -> Result<Url, url::ParseError> {
        match self {
            Self::Av(av) => format!("{}?aid={}", CID_API, av),
            Self::Bv(bv) => format!("{}?bvid={}", CID_API, bv),
            Self::Ep(ep) => format!("{}?ep_id={}", BANGUMI_CID_API, ep),
            Self::Ss(ss) => format!("{}?season_id={}", BANGUMI_CID_API, ss),
        }
        .parse()
    }
    fn as_video_api(&self, cid: u64) -> Result<Url, url::ParseError> {
        match self {
            Id::Av(ref avid) => format!("{}?cid={}&fnval=16&fourk=1&avid={}", VIDEO_API, cid, avid),
            Id::Bv(ref bvid) => format!("{}?cid={}&fnval=16&fourk=1&bvid={}", VIDEO_API, cid, bvid),
            _ => unimplemented!(),
        }
        .parse()
    }
}

impl BaseExtractor {
    fn as_video_api(&self) -> String {
        format!(
            "{}?cid={}&qn=127&fnval=2000&fourk=1&avid={}",
            VIDEO_API, self.cid, self.aid
        )
    }
    pub fn new(aid: u64, cid: u64, client: Client) -> Self {
        Self { aid, cid, client }
    }
    pub async fn title(&mut self) -> Result<String, Error> {
        let url = format!("{}?aid={}", VIDEO_INFO_API, self.aid).parse()?;
        let info = self.client.send_json_request(url).await?;
        match info["data"]["title"] {
            Value::String(ref s) => Ok(s.to_owned()),
            _ => err::InvalidResponse { resp: info }.fail(),
        }
    }
}

#[async_trait::async_trait]
impl Extract for BaseExtractor {
    async fn extract(&mut self) -> crate::FinaResult {
        let url = self.as_video_api();
        let data = self.client.send_json_request(Url::parse(&url)?).await?;
        let tracks = match parse_dash(&data["data"]["dash"])
            .transpose()
            .or_else(|| parse_durl(&data["data"]["durl"]).transpose())
        {
            Some(Ok(tracks)) => tracks,
            Some(Err(_)) | None => return err::InvalidResponse { resp: data }.fail(),
        };
        let origin = Origin::new(tracks, String::new());
        let title = self.title().await.unwrap_or_default();
        Ok(Finata::new(vec![origin], title))
    }
}

impl Video {
    pub fn new(s: &str) -> Result<Self, Error> {
        let url: Url = Url::parse(s)?;
        let id = Id::from_url(&url)?;
        let page = url
            .query_pairs()
            .find_map(|(key, v)| if key == "p" { v.parse().ok() } else { None });
        match id {
            Id::Av(_) | Id::Bv(_) => Ok(Self::with_id(id, page)),
            _ => Err(Error::InvalidUrl { url }),
        }
    }
    pub fn with_id(id: Id, page: Option<usize>) -> Self {
        Self::with_client(Client::with_header(HEADERS.clone()), id, page)
    }
    pub fn with_client(client: Client, id: Id, page: Option<usize>) -> Self {
        Self { client, id, page }
    }
    pub async fn playlist_json(&self) -> Result<Vec<Value>, Error> {
        let url = self.id.as_cid_api()?;
        let data = self.client.send_json_request(url).await?;
        match data["data"] {
            Value::Array(ref cids) => Ok(cids.clone()),
            _ => err::InvalidResponse { resp: data }.fail(),
        }
    }
    /// Returns dash url
    // todo: move this method into `BaseExtractor`
    pub async fn video_url_json(&self, cid: u64) -> Result<Value, Error> {
        let url = self.id.as_video_api(cid)?;
        self.client.send_json_request(url).await
    }
    // todo: move this method into `BaseExtractor`
    pub async fn video_dash_urls(&self, cid: u64) -> Result<Value, Error> {
        let mut data = self.video_url_json(cid).await?;
        match &mut data["data"]["dash"] {
            Value::Null => err::InvalidResponse { resp: data }.fail(),
            res => Ok(res.take()),
        }
    }
    pub async fn video_info_json(&self) -> Result<Value, Error> {
        let url = match &self.id {
            Id::Av(av) => format!("{}?aid={}", VIDEO_INFO_API, av),
            Id::Bv(bv) => format!("{}?bvid={}", VIDEO_INFO_API, bv),
            _ => unreachable!(),
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
        let info = self.video_info_json().await?;
        let aid = match &info["data"]["aid"] {
            Value::Number(n) => Ok(n.as_u64().expect("Bad aid")),
            _ => err::InvalidResponse { resp: info }.fail(),
        }?;
        let cid = extract_cid(&page)?;
        let mut base_extor = BaseExtractor::new(aid, cid, self.client.clone());
        base_extor.extract().await
    }
}

impl AsClient for Video {
    fn client(&self) -> &Client {
        &self.client
    }
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

impl Bangumi {
    pub fn with_id(id: Id) -> Self {
        Self::with_client(Client::with_header(HEADERS.clone()), id)
    }
    pub fn with_client(client: Client, id: Id) -> Self {
        Self { client, id }
    }
    pub fn new(s: &str) -> Result<Self, Error> {
        let url: Url = Url::parse(s)?;
        let id = Id::from_url(&url)?;
        match id {
            Id::Ep(_) | Id::Ss(_) => Ok(Self::with_id(id)),
            _ => Err(Error::InvalidUrl { url }),
        }
    }
    #[deprecated = "this method does not return a complete playlist"]
    pub async fn playlist_json(&self) -> Result<Vec<Value>, Error> {
        let url = self.id.as_cid_api()?;
        let data = self.client.send_json_request(url).await?;
        match data["result"]["episodes"] {
            Value::Array(ref eps) => Ok(eps.clone()),
            _ => err::InvalidResponse { resp: data }.fail(),
        }
    }
    async fn current_page(&self) -> Result<Value, Error> {
        let url = self.id.as_cid_api()?;
        let data = self.client.send_json_request(url).await?;
        let mut playlist = Vec::new();
        if let Value::Array(ref eps) = data["result"]["episodes"] {
            playlist.extend_from_slice(eps);
        }
        // only main episodes lie in data["result"]["episodes"]
        if let Value::Array(ref sections) = data["result"]["section"] {
            for section in sections {
                if let Value::Array(ref eps) = section["episodes"] {
                    playlist.extend_from_slice(eps);
                }
            }
        }
        let page = match self.id {
            Id::Ss(_) => playlist.first(),
            Id::Ep(epid) => playlist.iter().find(|ep| ep["id"].as_u64() == Some(epid)),
            _ => unreachable!(),
        };
        match page {
            Some(res) => Ok(res.to_owned()),
            None => err::InvalidResponse { resp: data }.fail(),
        }
    }
}

#[async_trait::async_trait]
impl Extract for Bangumi {
    async fn extract(&mut self) -> crate::FinaResult {
        let page = self.current_page().await?;
        let aid = page["aid"].as_u64().unwrap();
        let cid = page["cid"].as_u64().unwrap();
        let mut base_extor = BaseExtractor::new(aid, cid, self.client.clone());
        base_extor.extract().await
    }
}

impl AsClient for Bangumi {
    fn client(&self) -> &Client {
        &self.client
    }
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

impl BaseLiveExtractor {
    pub fn new(cid: u64, client: Client) -> Self {
        Self { cid, client }
    }

    pub fn as_live_url(&self) -> String {
        let cid = self.cid;
        format!("{LIVE_API}?cid={cid}&platform=h5&quality=4")
    }

    async fn _extract(&mut self) -> crate::FinaResult {
        let url = self.as_live_url();
        let data = self.client.send_json_request(url.parse()?).await?;
        let live_url = match data["data"]["durl"][0]["url"] {
            Value::String(ref s) => Url::parse(s)?,
            _ => return err::InvalidResponse { resp: data }.fail(),
        };
        let origin = Origin::video(live_url, String::new());
        Ok(Finata::new(vec![origin], String::new()))
    }
}

#[async_trait::async_trait]
impl Extract for BaseLiveExtractor {
    async fn extract(&mut self) -> crate::FinaResult {
        self._extract().await
    }
}

impl AsClient for BaseLiveExtractor {
    fn client(&self) -> &Client {
        &self.client
    }
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

impl Live {
    pub fn new(s: &str) -> Result<Self, Error> {
        let url = Url::parse(s)?;
        let id = url
            .path_segments()
            .and_then(|it| it.filter(|p| !p.is_empty()).next_back())
            .ok_or_else(|| err::InvalidUrl { url: url.clone() }.build())?
            .parse()
            .map_err(|_| err::InvalidUrl { url }.build())?;
        Ok(Self {
            id,
            client: Client::new(),
        })
    }

    pub fn as_info_url(&self) -> String {
        let id = self.id;
        format!("{LIVE_INFO_API}?id={id}")
    }

    async fn title(&self, uid: u64) -> Result<String, Error> {
        let url = format!("{SPACE_INFO_API}?mid={uid}");
        let data = self.client.send_json_request(url.parse()?).await?;
        match data["data"]["live_room"]["title"] {
            Value::String(ref title) => Ok(title.to_owned()),
            _ => err::InvalidResponse { resp: data }.fail(),
        }
    }

    async fn _extract(&self) -> crate::FinaResult {
        let url = self.as_info_url();
        let data = self.client.send_json_request(url.parse()?).await?;
        let cid = match data["data"]["room_id"] {
            Value::Number(ref cid) if cid.is_u64() => cid.as_u64().unwrap(),
            _ => return err::InvalidResponse { resp: data }.fail(),
        };
        let uid = match data["data"]["uid"] {
            Value::Number(ref uid) if uid.is_u64() => uid.as_u64().unwrap(),
            _ => return err::InvalidResponse { resp: data }.fail(),
        };
        let mut base_extor = BaseLiveExtractor::new(cid, self.client.clone());
        let (raw, _) = base_extor.extract().await?.into_parts();
        Ok(Finata::new(raw, self.title(uid).await?))
    }
}

#[async_trait::async_trait]
impl Extract for Live {
    async fn extract(&mut self) -> crate::FinaResult {
        self._extract().await
    }
}

impl AsClient for Live {
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
        .ok_or_else(|| err::InvalidResponse { resp: data.clone() }.build())
}

fn parse_dash(info: &Value) -> Result<Option<Vec<Track>>, Error> {
    let video_url = info["video"]
        .as_array()
        .and_then(|data| data.iter().find_map(|data| data["baseUrl"].as_str()));
    let audio_url = info["audio"]
        .as_array()
        .and_then(|data| data.iter().find_map(|data| data["baseUrl"].as_str()));
    Ok(match (video_url, audio_url) {
        (Some(vurl), Some(aurl)) => Some(vec![
            Track::Video(vurl.parse()?),
            Track::Audio(aurl.parse()?),
        ]),
        (Some(url), _) => Some(vec![Track::Video(url.parse()?)]),
        (_, Some(url)) => Some(vec![Track::Audio(url.parse()?)]),
        _ => None,
    })
}

fn parse_durl(info: &Value) -> Result<Option<Vec<Track>>, Error> {
    Ok(match info.as_array() {
        Some(urls) => {
            let mut tracks = Vec::with_capacity(urls.len());
            // need sorting by order probably
            for i in urls {
                match &i["url"] {
                    Value::String(url) => tracks.push(Track::Video(url.parse()?)),
                    _ => return Ok(None),
                }
            }
            Some(tracks)
        }
        None => None,
    })
}
