use reqwest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("RequestError: {0}")]
    Request(#[from] reqwest::Error),
    #[error("InvalidInput: {0}")]
    InvalidInput(String),
    #[error("Unexpect None")]
    None,
    #[error("InvalidUrl")]
    InvalidUrl(#[from] url::ParseError),
}

impl Error {
    pub fn invalid_input_url<S: ToString>(s: S) -> Self {
        Self::InvalidInput(s.to_string())
    }
}