use url::Url;

pub mod error;
pub mod utils;
pub mod website;

pub use crate::error::*;

pub type FinaResult = Result<Box<dyn Iterator<Item = Result<Finata, Error>>>, Error>;

#[async_trait::async_trait]
pub trait Extract {
    async fn extract(&mut self) -> Result<Box<dyn Iterator<Item = Result<Finata, Error>>>, Error>;
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
    raw: Url,
    format: Format,
    title: String,
}

impl Finata {}
