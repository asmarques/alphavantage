use derive_more::Display;

/// Set of errors which can occur when calling the API.
#[derive(Display, Debug)]
pub enum Error {
    /// Error establishing a network connection.
    #[display(fmt = "connection error: {}", _0)]
    ConnectionError(String),
    /// HTTP error returned by the API.
    #[display(fmt = "server returned HTTP status code {}", _0)]
    ServerError(u16),
    /// Error parsing the API response.
    #[display(fmt = "parsing error: {}", _0)]
    ParsingError(String),
    /// Error returned by the API.
    #[display(fmt = "API error: {}", _0)]
    APIError(String),
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(inner: reqwest::Error) -> Error {
        Error::ConnectionError(inner.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(inner: serde_json::Error) -> Error {
        Error::ParsingError(inner.to_string())
    }
}

impl From<chrono::format::ParseError> for Error {
    fn from(inner: chrono::format::ParseError) -> Error {
        Error::ParsingError(inner.to_string())
    }
}
