pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("transport error")]
    Transport(#[from] reqwest::Error),
    #[error("bad url")]
    BadUrl(#[from] url::ParseError),
    #[error("missing header")]
    MissingHeader(Option<reqwest::header::HeaderName>),
}
