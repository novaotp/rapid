use core::fmt;
use std::collections::HashMap;

use crate::status_code::StatusCode;

/// A valid body payload for a response.
pub trait ResponseBodyPayload: std::fmt::Debug {
    /// A string representation of the payload.
    fn as_str(&self) -> &str;

    /// The `Content-Type` of the payload.
    fn get_content_type(&self) -> &str;

    /// Whether the body payload is empty or not.
    fn is_empty(&self) -> bool;

    /// The length of the body payload when sent.
    fn len(&self) -> usize;
}

/// An HTTP response.
#[derive(Debug)]
pub struct Response {
    pub version: String,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Option<Box<dyn ResponseBodyPayload>>,
}

impl Response {
    /// Sets a header, overriding any existing value if the key already exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use prime_http::response::Response;
    /// let mut response = Response::default();
    ///
    /// response.set_header("Content-Type", "text/plain");
    /// assert_eq!(response.headers.get("Content-Type"), Some("text/plain"));
    ///
    /// response.set_header("Content-Type", "application/json");
    /// assert_eq!(response.headers.get("Content-Type"), Some("application/json"));
    /// ```
    pub fn set_header(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.headers.insert(key.into(), val.into());
    }

    /// Sets the body of the response.
    ///
    /// If the following headers are not set, they will be inferred from the new body :
    /// - `Content-Type`
    /// - `Content-Length`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use prime_http::response::Response;
    /// let mut response = Response::default();
    ///
    /// assert_eq!(response.headers.get("Content-Type"), None);
    /// response.set_body()
    /// ```
    pub fn set_body(&mut self, body: Box<dyn ResponseBodyPayload>) {
        self.body = Some(body);

        if let Some(body) = &self.body {
            self.headers
                .entry(String::from("Content-Type"))
                .or_insert(body.get_content_type().to_owned());

            self.headers
                .entry(String::from("Content-Length"))
                .or_insert(body.len().to_string());
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Response {
            version: String::from("HTTP/1.1"),
            status: StatusCode::default(),
            headers: HashMap::new(),
            body: None,
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}\r\n",
            self.version,
            self.status.code(),
            self.status.as_str()
        )?;

        for (key, val) in &self.headers {
            write!(f, "{}: {}\r\n", key, val)?;
        }

        write!(f, "\r\n")?;

        if let Some(body) = &self.body {
            write!(f, "{}", body.as_str())?;
        }

        write!(f, "\r\n")
    }
}
