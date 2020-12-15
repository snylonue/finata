use serde_json::Value;
use snafu::Snafu;
use url::Url;

pub use NetworkError as a;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(context(false))]
    ParseUrlError {
        source: url::ParseError,
    },
    InvalidUrl {
        url: Url,
    },
    InvalidResponse {
        resp: Value,
    },
    #[snafu(display("Fails to fetch `{}`: {}", url, source))]
    NetworkError {
        url: Url,
        source: reqwest::Error,
    },
}



#[derive(Debug, Snafu)]
pub struct FinataError {
    kind: Error,
}

impl From<Error> for FinataError {
    fn from(_: Error) -> Self {
        todo!()
    }
}
