use reqwest;
use thiserror::Error;
use std::mem::replace;

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
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Errors: {0:#?}")]
    Errors(Vec<Self>),
}

impl Error {
    pub fn invalid_input_url<S: ToString>(s: S) -> Self {
        Self::InvalidInput(s.to_string())
    }
    pub fn needs_vip() -> Self {
        Self::PermissionDenied(String::from("Vip account is needed"))
    }
    pub fn push(&mut self, err: Self) {
        match self {
            Self::Errors(e) => e.push(err),
            _ => {
                let e = replace(self, Self::Errors(Vec::with_capacity(2)));
                self.push(e);
                self.push(err);
            }
        }
    }
    pub const fn with_errors(errs: Vec<Self>) -> Self {
        Self::Errors(errs)
    }
}