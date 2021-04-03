use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use url::Url;
use utils::Client;

pub mod error;
pub mod utils;
pub mod website;

pub use crate::error::*;

pub type FinaResult<T = Finata> = Result<T, Error>;

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

#[async_trait::async_trait]
pub trait Extract {
    async fn extract(&mut self) -> FinaResult;
}

pub trait Config {
    fn client(&self) -> &Client;
    fn client_mut(&mut self) -> &mut Client;
}

pub trait ExtractSync {
    fn extract_sync(&mut self) -> FinaResult;
}

impl<T: Extract> ExtractSync for T {
    fn extract_sync(&mut self) -> FinaResult {
        RUNTIME.block_on(async { self.extract().await })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Track {
    Video(Url),
    Audio(Url),
    Text(Url),
    Image(Url),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Origin {
    pub tracks: Vec<Track>,
    pub title: String,
}

#[derive(Debug, PartialEq)]
pub struct Finata {
    raws: Vec<Origin>,
    title: String,
}

impl Origin {
    pub fn new(tracks: Vec<Track>, title: String) -> Self {
        Self { tracks, title }
    }
    pub fn video(url: Url, title: String) -> Self {
        Self::new(vec![Track::Video(url)], title)
    }
    pub fn audio(url: Url, title: String) -> Self {
        Self::new(vec![Track::Audio(url)], title)
    }
}

impl Finata {
    pub fn new(raws: Vec<Origin>, title: String) -> Self {
        Self { raws, title }
    }
    pub fn raws(&self) -> &[Origin] {
        &self.raws
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn into_title(self) -> String {
        self.title
    }
    pub fn into_parts(self) -> (Vec<Origin>, String) {
        (self.raws, self.title)
    }
}
