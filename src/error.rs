use failure_derive::Fail;

/// Set of errors which can occur when calling the API.
#[derive(Debug, Fail)]
pub enum Error {
    /// Error establishing a network connection.
    #[fail(display = "connection error: {}", error)]
    ConnectionError {
        /// Internal error.
        #[cause]
        error: failure::Compat<failure::Error>,
    },
    /// HTTP error returned by the API.
    #[fail(display = "server returned HTTP status code {}", code)]
    ServerError {
        /// HTTP error code.
        code: u16,
    },
    /// Error parsing the result returned from the API.
    #[fail(display = "parsing error: {}", error)]
    ParsingError {
        /// Internal error.
        #[cause]
        error: failure::Compat<failure::Error>,
    },
}
