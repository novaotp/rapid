use prime_http::response::ResponseBodyPayload;

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

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
