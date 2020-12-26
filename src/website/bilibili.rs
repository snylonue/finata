use crate::utils;
use crate::utils::Client;
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
    /// add ?bvid={} or ?aid={}
    static ref CID_API: Url = Url::parse("https://api.bilibili.com/x/player/pagelist").unwrap();
    /// add ?cid={}&qn={}&avid={} or ?cid={}&qn={}&bvid={}
    static ref VIDEO_API: Url = Url::parse("https://api.bilibili.com/x/player/playurl").unwrap();
}

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
    pub async fn playlist_json(&self) -> Result<Value, Error> {
        let url = match &self.id {
            Id::Av(av) => {
                let mut url = CID_API.clone();
                url.set_query(Some(&format!("aid={}", av)));
                url
            }
            Id::Bv(bv) => {
                let mut url = CID_API.clone();
                url.set_query(Some(&format!("bvid={}", bv)));
                url
            }
        };
        self.client.send_json_request(url).await
    }
}

#[async_trait::async_trait]
impl Extract for Bilibili {
    async fn extract(&mut self) -> Result<Box<dyn Iterator<Item = Result<Finata, Error>>>, Error> {
        todo!()
    }
}
