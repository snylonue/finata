use crate::{
    error as err, utils, utils::Client, AsClient, Error, Extract, Origin, Playlist, Track,
};
use once_cell::sync::Lazy;
use reqwest::{header, header::HeaderMap};
use serde_json::Value;
use url::Url;

const LIMIT: u64 = 96;

static HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://www.pixiv.net/",
    }
});
static IMAGE_API: Lazy<Url> =
    Lazy::new(|| Url::parse("https://www.pixiv.net/ajax/illust/").unwrap());

#[derive(Debug)]
pub struct Pixiv {
    client: Client,
    pid: String,
}

/// A extractor for pixiv user collection (bookmark).
/// Works only when cookies are passed.
pub struct Collection {
    client: Client,
    uid: String,
}

impl Pixiv {
    pub fn new(s: &str) -> Result<Self, Error> {
        let url: Url = Url::parse(s.trim_end_matches('/'))?;
        let pid = url
            .path_segments()
            .ok_or_else(|| Error::InvalidUrl { url: url.clone() })?
            .next_back()
            .ok_or_else(|| Error::InvalidUrl { url: url.clone() })?
            .to_owned();
        Ok(Self::with_pid(pid))
    }
    pub fn with_pid(pid: String) -> Self {
        Self::with_client(Client::with_header(HEADERS.clone()), pid)
    }
    pub fn with_client(client: Client, pid: String) -> Self {
        Self { client, pid }
    }
    async fn raw_url_json(&self) -> Result<Value, Error> {
        let url = IMAGE_API.join(&format!("{}/pages", self.pid)).unwrap();
        let data = self.client.send_json_request(url).await?;
        Ok(data)
    }
    async fn meta_json(&self) -> Result<Value, Error> {
        let url = IMAGE_API.join(&self.pid).unwrap();
        let data = self.client.send_json_request(url).await?;
        Ok(data)
    }
    async fn raw_urls(&self) -> Result<Vec<Value>, Error> {
        let mut data = self.raw_url_json().await?;
        match data["body"] {
            Value::Array(ref mut urls) => Ok(urls.to_owned()),
            _ => err::InvalidResponse { resp: data }.fail(),
        }
    }
    async fn title(&self) -> Result<String, Error> {
        let data = self.meta_json().await?;
        match data["body"]["title"] {
            Value::String(ref title) => Ok(title.to_owned()),
            _ => err::InvalidResponse { resp: data }.fail(),
        }
    }
}

#[async_trait::async_trait]
impl Extract for Pixiv {
    async fn extract(&mut self) -> crate::FinaResult {
        let title = self.title().await?;
        let urls = self.raw_urls().await?;
        let tracks = urls
            .into_iter()
            .map(move |v| {
                let url_data = &v["urls"]["original"];
                match url_data {
                    Value::String(ref url) => Url::parse(url).map_err(Into::into).map(Track::Image),
                    _ => err::InvalidResponse { resp: v }.fail(),
                }
            })
            .collect::<Result<_, Error>>()?;
        let raws = Origin::new(tracks, title.clone());
        Ok(Playlist::new(vec![raws], title))
    }
}

impl AsClient for Pixiv {
    fn client(&self) -> &Client {
        &self.client
    }
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

impl Collection {
    pub fn new(s: &str) -> Result<Self, Error> {
        let url: Url = Url::parse(s.trim_end_matches('/'))?;
        let uid = url
            .path_segments()
            .ok_or_else(|| Error::InvalidUrl {
                url: url.to_owned(),
            })?
            .nth_back(3)
            .ok_or_else(|| Error::InvalidUrl {
                url: url.to_owned(),
            })?
            .to_owned();
        Ok(Self::with_uid(uid))
    }
    pub fn with_uid(uid: String) -> Self {
        Self::with_client(Client::with_header(HEADERS.clone()), uid)
    }
    pub fn with_client(client: Client, uid: String) -> Self {
        Self { client, uid }
    }
}

#[async_trait::async_trait]
impl Extract for Collection {
    async fn extract(&mut self) -> crate::FinaResult {
        let base_url = format!(
            "https://www.pixiv.net/ajax/user/{}/illusts/bookmarks",
            self.uid
        );
        let mut origins = Vec::<Origin>::new();
        let mut offset = 0u64;
        loop {
            let url = Url::parse_with_params(
                &base_url,
                [
                    ("tag", ""),
                    ("offset", &offset.to_string()),
                    ("limit", &LIMIT.to_string()),
                    ("rest", "show"),
                ],
            )
            .unwrap();
            let coll = self.client().send_json_request(url).await?;
            match &coll["body"]["works"] {
                Value::Array(works) => {
                    for work in works {
                        let id = match work["id"].as_str() {
                            Some(id) => id,
                            _ => return err::InvalidResponse { resp: coll }.fail(),
                        };
                        // ignore invalid illusts
                        let (extraction, _) = Pixiv::with_pid(id.to_owned())
                            .extract()
                            .await
                            .unwrap_or_default()
                            .into_parts();
                        origins.extend(extraction);
                    }
                }
                _ => return err::InvalidResponse { resp: coll }.fail(),
            }
            // if total is unavaliable or less than offset, return
            if offset >= coll["body"]["total"].as_u64().unwrap_or(0) {
                break Ok(Playlist::new(origins, String::new()));
            }
            offset += LIMIT;
        }
    }
}

impl AsClient for Collection {
    fn client(&self) -> &Client {
        &self.client
    }
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}
