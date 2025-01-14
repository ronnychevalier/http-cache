use miette::Diagnostic;
use thiserror::Error;

/// A generic “error” for HTTP caches
#[derive(Error, Diagnostic, Debug)]
pub enum CacheError {
    /// A general error used as a catch all for other errors via anyhow
    #[error(transparent)]
    #[diagnostic(code(http_cache::general))]
    General(#[from] anyhow::Error),
    /// Error from http
    #[error(transparent)]
    #[diagnostic(code(http_cache::http_error))]
    HttpError(#[from] http::Error),
    /// There was an error parsing the HTTP status code
    #[error(transparent)]
    #[diagnostic(code(http_cache::invalid_status_code))]
    InvalidStatusCode(#[from] http::status::InvalidStatusCode),
    /// There was an error converting the header to a string
    #[error(transparent)]
    #[diagnostic(code(http_cache::header_to_str_error))]
    HeaderToStrError(#[from] http::header::ToStrError),
    /// There was an error parsing the HTTP method
    #[error(transparent)]
    #[diagnostic(code(http_cache::invalid_method))]
    InvalidMethod(#[from] http::method::InvalidMethod),
    /// There was an error parsing the URI
    #[error(transparent)]
    #[diagnostic(code(http_cache::invalid_uri))]
    InvalidUri(#[from] http::uri::InvalidUri),
    /// There was an error parsing an HTTP header value
    #[error(transparent)]
    #[diagnostic(code(http_cache::invalid_header_value))]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    /// There was an error parsing an HTTP header name
    #[error(transparent)]
    #[diagnostic(code(http_cache::invalid_header_name))]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    /// Error from reqwest
    #[cfg(feature = "client-reqwest")]
    #[error(transparent)]
    #[diagnostic(code(http_cache::reqwest_error))]
    ReqwestError(#[from] reqwest::Error),
    /// Error from reqwest_middleware
    #[cfg(feature = "client-reqwest")]
    #[error(transparent)]
    #[diagnostic(code(http_cache::reqwest_middleware_error))]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    /// Error from cacache
    #[cfg(feature = "manager-cacache")]
    #[error(transparent)]
    #[diagnostic(code(http_cache::cacache_error))]
    CaCacheError(#[from] cacache::Error),
    /// Error from bincode
    #[cfg(feature = "manager-cacache")]
    #[error(transparent)]
    #[diagnostic(code(http_cache::bincode_error))]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    /// There was an error parsing the HTTP request version
    #[error("Unknown HTTP version")]
    #[diagnostic(code(http_cache::bad_version))]
    BadVersion,
    /// There was an error parsing an HTTP header value
    #[error("Error parsing header value")]
    #[diagnostic(code(http_cache::bad_header))]
    BadHeader,
    /// There was an error parsing the HTTP request
    #[error(
        "Request object is not cloneable. Are you passing a streaming body?"
    )]
    #[diagnostic(code(http_cache::bad_request))]
    BadRequest,
}
