use reqwest::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    IO(#[from] std::io::Error),
    #[error("transport error")]
    Transport(#[from] reqwest::Error),
    #[error("bad url")]
    BadUrl(#[from] url::ParseError),
    #[error("missing header")]
    MissingHeader(Option<reqwest::header::HeaderName>),
    #[error("HTTP status {status_code}, code: {error_code}, message: {error_message}")]
    Endpoint {
        status_code: StatusCode,
        error_code: String,
        error_message: String,
    },
}