use url::Url;

pub mod website;
pub mod utils;
pub mod error;

use snafu::Snafu;
use serde_json::Value;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(context(false))]
    ParseUrlError { source: url::ParseError },
    InvalidUrl { url: Url },
    InvalidResponse { resp: Value },
    #[snafu(display("Fails to fetch `{}`: {}", url, source))]
    NetworkError {
        url: Url,
        source: reqwest::Error,
    }
}

pub type NetWorkError<T> = NetworkError<T>;
pub type InValidUrl<T> = InvalidUrl<T>;
pub type InValidResponse<T> = InvalidResponse<T>;

#[derive(Debug, Snafu)]
pub struct FinataError {
    kind: Error,
}

impl From<Error> for FinataError {
    fn from(_: Error) -> Self {
        todo!()
    }
}

#[async_trait::async_trait]
pub trait Extract {
    async fn extract(&mut self) -> Box<dyn Iterator<Item=Result<Finata, FinataError>>>;
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

impl Finata {
}