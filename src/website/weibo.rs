use once_cell::sync::Lazy;
use reqwest::{
    header::{self, HeaderMap},
    Url,
};
use serde::Deserialize;

use crate::{
    utils::{self, Client},
    Error, Extract, FinaResult, Finata, Origin, Track,
};

static HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    crate::hdmap! {
        header::USER_AGENT => utils::UA.clone(),
        header::REFERER => "https://weibo.com",
    }
});

pub struct Live {
    id: String,
    client: Client,
}

impl Live {
    pub fn new(url: &str) -> Result<Self, Error> {
        let url = Url::parse(url)?;
        match url
            .path_segments()
            .map(|p| p.filter(|s| !s.is_empty()).next_back())
            .flatten()
        {
            Some(id) => Ok(Self {
                id: dbg!(id).to_string(),
                client: Client::with_header(HEADERS.clone()),
            }),
            None => Err(Error::InvalidUrl { url }),
        }
    }

    async fn _extract(&self) -> FinaResult {
        let url = format!(
            "https://weibo.com/l/!/2/wblive/room/show_pc_live.json?live_id={}",
            self.id
        );
        let response = self
            .client
            .send_json_request_generic::<Response>(url.parse()?)
            .await?;
        let pl = self
            .client
            .client()
            .get(response.data.replay_origin_url)
            .send()
            .await?
            .text()
            .await?;
        let vi_list = pl
            .lines()
            .filter(|s| !s.starts_with('#'))
            .map(|s| format!("https://live.video.weibocdn.com/{}", s));
        let tracks = vi_list
            .map(|url| -> Result<_, Error> { Ok(Track::Video(url.parse()?)) })
            .collect::<Result<_, Error>>()?;
        let origin = Origin::new(tracks, String::new());
        Ok(Finata::new(vec![origin], response.data.title))
    }
}
#[async_trait::async_trait]
impl Extract for Live {
    async fn extract(&mut self) -> crate::FinaResult {
        self._extract().await
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ResponseData {
    pub replay_origin_url: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Response {
    pub data: ResponseData,
}
