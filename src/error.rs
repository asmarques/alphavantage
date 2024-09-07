/// Set of errors which can occur when calling the API.
#[derive(Debug)]
pub enum Error {
    /// Error establishing a network connection.
    ConnectionError(String),
    /// HTTP error returned by the API.
    ServerError(u16),
    /// Error parsing the API response.
    ParsingError(String),
    /// Error returned by the API.
    APIError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectionError(e) => write!(f, "connection error: {}", e),
            Error::ServerError(e) => write!(f, "server returned HTTP status code {}", e),
            Error::ParsingError(e) => write!(f, "parsing error: {}", e),
            Error::APIError(e) => write!(f, "API error: {}", e),
        }
    }
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
