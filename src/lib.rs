use url::Url;
use utils::Client;

pub mod error;
pub mod utils;
pub mod website;

pub use crate::error::*;

pub type FinaResult<T = Finata> = Result<T, Error>;

#[async_trait::async_trait]
pub trait Extract {
    async fn extract(&mut self) -> FinaResult;
    fn client(&self) -> &Client;
    fn client_mut(&mut self) -> &mut Client;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Format {
    Video,
    Audio,
    Text,
    Image,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Origin {
    pub format: Format,
    pub url: Url,
}

impl Origin {
    pub fn new(format: Format, url: Url) -> Self {
        Self { format, url }
    }
}

#[derive(Debug, PartialEq)]
pub struct Finata {
    raws: Vec<Origin>,
    title: String,
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
}
