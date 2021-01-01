use url::Url;

pub mod error;
pub mod utils;
pub mod website;

pub use crate::error::*;

pub type FinaResult = Result<Finata, Error>;

#[async_trait::async_trait]
pub trait Extract {
    async fn extract(&mut self) -> Result<Finata, Error>;
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
}
