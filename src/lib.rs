pub mod website;

use reqwest::Url;
use reqwest::header::HeaderMap;

#[derive(Debug, PartialEq)]
pub enum Format {
    Video,
    Audio,
    Text,
    Image,
}
#[derive(Debug, PartialEq)]
pub struct FinataData {
    pub url: Url,
    pub raw_url: Vec<(Url, Format)>,
    pub header: HeaderMap,
    pub title: Option<String>,
}

impl FinataData {
    pub const fn new(url: Url, raw_url: Vec<(Url, Format)>, header: HeaderMap, title: Option<String>) -> Self {
        Self { url, raw_url, header, title }
    }
}