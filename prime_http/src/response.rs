use std::collections::HashMap;

use crate::status_code::StatusCode;

/// A valid body payload for a response.
pub trait ResponseBodyPayload: std::fmt::Debug + Clone {
    /// A string representation of the payload.
    fn as_str(&self) -> &str;

    /// The `Content-Type` of the payload.
    fn get_content_type(&self) -> &str;

    /// Converts the payload to an optional boxed payload.
    fn as_body(&self) -> Option<Box<Self>>;
}

#[derive(Debug)]
pub struct StringResponseBodyPayload(pub String);

impl StringResponseBodyPayload {
    pub fn new(payload: impl Into<String>) -> Self {
        StringResponseBodyPayload(payload.into())
    }
}

impl ResponseBodyPayload for StringResponseBodyPayload {
    fn get_content_type(&self) -> &str {
        "text/plain"
    }

    fn as_str(&self) -> &str {
        &self.0
    }

    fn as_body(&self) -> Option<Box<Self>> {
        Some(Box::new(self.clone()))
    }
}

/// An HTTP response.
#[derive(Debug)]
pub struct Response<T: ResponseBodyPayload> {
    pub version: String,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Option<Box<T>>,
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
    /// If the `Content-Type` header is not set, it is inferred from the new body.
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
    pub fn set_body(&mut self, body: Option<Box<dyn ResponseBodyPayload>>) {
        self.body = body;

        if let Some(body) = &self.body {
            self.headers
                .entry("Content-Type".to_owned())
                .or_insert(body.content_type().to_owned());
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
