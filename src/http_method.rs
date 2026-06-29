use std::str::FromStr;

/// An error struct when an HTTP method is invalid.
///
/// The invalid method is stored.
#[derive(Debug, PartialEq)]
pub struct InvalidHttpMethodError(pub String);

/// All valid and supported HTTP methods.
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum HttpMethod {
    HEAD,
    GET,
    QUERY,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl FromStr for HttpMethod {
    type Err = InvalidHttpMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HEAD" => Ok(HttpMethod::HEAD),
            "GET" => Ok(HttpMethod::GET),
            "QUERY" => Ok(HttpMethod::QUERY),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "PATCH" => Ok(HttpMethod::PATCH),
            "DELETE" => Ok(HttpMethod::DELETE),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "CONNECT" => Ok(HttpMethod::CONNECT),
            "TRACE" => Ok(HttpMethod::TRACE),
            _ => Err(InvalidHttpMethodError(s.to_owned())),
        }
    }
}
