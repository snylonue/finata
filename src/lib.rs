pub mod error;
pub mod utils;
pub mod website;

pub use crate::error::Error;
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::Url;

#[async_trait]
pub trait Extract {
    // type Iter: Iterator<Item = Result<Finata, Error>>;

    async fn extract(&mut self) -> Box<dyn Iterator<Item = Result<Finata, Error>>>;
    fn header(&self) -> HeaderMap;
}

#[derive(Debug, PartialEq)]
pub enum Format {
    Video,
    Audio,
    Text,
    Image,
}
#[derive(Debug, PartialEq)]
pub struct Finata {
    pub url: Url,
    pub raw: Url,
    pub format: Format,
    pub title: Option<String>,
}

impl Finata {
    pub const fn new(url: Url, raw: Url, format: Format, title: Option<String>) -> Self {
        Self {
            url,
            raw,
            format,
            title,
        }
    }
}
