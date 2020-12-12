use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    Pixiv {
        source: crate::website::pixiv::Error,
    },
}

impl From<crate::website::pixiv::Error> for Error {
    fn from(e: crate::website::pixiv::Error) -> Self {
        Self::Pixiv { source: e }
    }
}
