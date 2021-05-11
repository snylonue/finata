use serde_json::Value;
use snafu::Snafu;
use url::Url;

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
    #[snafu(display("InvalidResponse: {}", resp))]
    InvalidResponse {
        resp: Value,
    },
    #[snafu(display("Fails to fetch `{}`: {}", url, source))]
    NetworkError {
        url: Url,
        source: reqwest::Error,
    },
    #[snafu(context(false))]
    ParseJsonError {
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
