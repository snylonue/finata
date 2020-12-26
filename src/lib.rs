use url::Url;
use futures::stream::Stream;

pub mod error;
pub mod utils;
pub mod website;

pub use crate::error::*;

pub type FinaResult = Result<Box<dyn Stream<Item = Result<Finata, Error>> + Unpin>, Error>;

#[async_trait::async_trait]
pub trait Extract {
    async fn extract(&mut self) -> Result<Box<dyn Stream<Item = Result<Finata, Error>> + Unpin>, Error>;
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
