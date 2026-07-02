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

impl HttpMethod {
    /// Whether this method allows a body to be present and used or not.
    ///
    /// If `false`, the body should be ignored, even if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use prime_server::http_method::HttpMethod;
    /// assert_eq!(HttpMethod::GET.allows_body(), false);
    /// assert_eq!(HttpMethod::POST.allows_body(), true);
    /// ```
    pub fn allows_body(&self) -> bool {
        matches!(
            self,
            HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH | HttpMethod::QUERY
        )
    }
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
