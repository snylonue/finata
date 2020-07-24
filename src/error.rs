use reqwest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("RequestError: {0}")]
    Request(#[from] reqwest::Error),
    #[error("InvaildInput: {0}")]
    InvaildInput(String),
    #[error("Unexpect None")]
    None,
}

impl Error {
    pub fn invaild_url<S: ToString>(s: &S) -> Self {
        Self::InvaildInput(s.to_string())
    }
}