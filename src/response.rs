use std::{
    collections::HashMap,
    fmt::{self, Write as _},
    io::{self, Write as _},
    net,
};

use crate::status::Status;

pub struct Response<'a> {
    stream: &'a net::TcpStream,
    headers: HashMap<String, String>,
    body: String,
    status: Status,
}

const HTTP_VERSION: &'static str = "HTTP/1.1";

pub enum SendError {
    Format(fmt::Error),
    IO(io::Error),
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<fmt::Error> for SendError {
    fn from(item: fmt::Error) -> Self {
        SendError::Format(item)
    }
}

impl From<io::Error> for SendError {
    fn from(item: io::Error) -> Self {
        SendError::IO(item)
    }
}

impl<'a> Response<'a> {
    pub fn new(stream: &'a net::TcpStream) -> Self {
        Response {
            stream,
            headers: HashMap::new(),
            body: String::new(),
            status: Status::OK,
        }
    }

    pub fn set_header(&mut self, key: String, val: String) -> () {
        self.headers.insert(key, val);
    }

    pub fn set_status(&mut self, status: Status) -> () {
        self.status = status;
    }

    pub fn set_body(&mut self, body: String) -> () {
        self.body = body;
    }

    /// Sends the response and closes the stream.
    ///
    /// An error occurs if a write is attempted after this call.
    pub fn send(&mut self) -> Result<(), SendError> {
        let write_response = self.build_write_response()?;
        self.stream.write_all(write_response.as_bytes())?;

        self.stream.flush()?;
        self.stream.shutdown(net::Shutdown::Write)?;

        Ok(())
    }

    /// Creates a response suitable to be written to the stream.
    fn build_write_response(&self) -> Result<String, SendError> {
        let mut w = String::new();
        writeln!(
            &mut w,
            "{} {} {}",
            HTTP_VERSION,
            self.status,
            self.status.as_string()
        )?;

        for (key, val) in &self.headers {
            writeln!(&mut w, "{}: {}", key, val)?;
        }

        writeln!(&mut w)?;

        writeln!(&mut w, "{}", self.body)?;
        writeln!(&mut w)?;

        Ok(w)
    }
}
