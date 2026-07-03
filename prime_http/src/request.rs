use core::fmt;
use std::{
    collections::HashMap,
    convert::Infallible,
    io::{self, BufRead as _, BufReader, Read as _},
    num::ParseIntError,
    str::FromStr,
    string::FromUtf8Error,
};

use crate::http_method::{HttpMethod, InvalidHttpMethodError};

#[derive(Debug)]
pub enum QueryValue {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum HeaderValue {
    Single(String),
    Many(Vec<String>),
}

impl fmt::Display for HeaderValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Single(s) => write!(f, "{}", s),
            Self::Many(vec) => write!(f, "{}", vec.join(", ")),
        }
    }
}

impl FromStr for HeaderValue {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<&str> = s.split(", ").collect();

        match values.len() {
            len if len > 1 => Ok(HeaderValue::Many(
                values.iter().map(|str| str.to_string()).collect(),
            )),
            _ => Ok(HeaderValue::Single(values[0].to_owned())),
        }
    }
}

/// An HTTP request.
#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub query: HashMap<String, QueryValue>,
    pub headers: HashMap<String, HeaderValue>,
    pub body: Option<String>,
}

impl Request {
    /// Parses an incoming HTTP request using a `BufReader`.
    ///
    /// For requests that have a body, one of two headers must appear :
    /// - `Transfert-Encoding` (which takes priority) but only supports the chunked value.
    /// - `Content-Length` which must be a valid integer.
    ///
    /// Otherwise, an empty body is assumed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use std::io::BufReader;
    /// # use std::net::TcpListener;
    /// # use prime_server::request::Request;
    /// # use prime_server::http_method::HttpMethod;
    /// # fn try_main() -> Result<(), prime_server::request::RequestParseError> {
    /// let data = b"POST / HTTP/1.1\r\n\
    ///              Content-Type: text/plain\r\n\
    ///              Content-Length: 11\r\n\
    ///              \r\n\
    ///              Hello World\r\n\
    ///              \r\n";
    /// let mut reader = BufReader::new(&data[..]);
    ///
    /// let request = Request::parse(&mut reader)?;
    ///
    /// assert_eq!(request.method, HttpMethod::POST);
    /// assert_eq!(request.path, String::from("/"));
    /// assert_eq!(request.version, String::from("HTTP/1.1"));
    /// assert_eq!(
    ///     request.headers,
    ///     HashMap::from([
    ///         (String::from("Content-Type"), String::from("text/plain")),
    ///         (String::from("Content-Length"), String::from("11"))
    ///     ])
    /// );
    /// assert_eq!(request.query, HashMap::new());
    /// assert_eq!(request.body, Some(String::from("Hello World")));
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// #    try_main().unwrap();
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// See [RequestParseError] for more details.
    pub fn parse<T: io::Read>(reader: &mut BufReader<T>) -> Result<Self, RequestParseError> {
        let (method, full_path, version) = read_start_line(reader)?;

        let (path, query_string) = match full_path.split_once("?") {
            Some((path, query_string)) => (path.to_owned(), query_string.to_owned()),
            None => (full_path.to_owned(), String::from("")),
        };

        let headers = read_headers(reader)?;
        let query = parse_query_string(&query_string);
        let body = get_body(reader, &method, &headers)?;

        Ok(Request {
            method,
            path,
            version,
            headers,
            query,
            body,
        })
    }
}

fn read_start_line<T: io::Read>(
    reader: &mut BufReader<T>,
) -> Result<(HttpMethod, String, String), RequestParseError> {
    let mut line = String::new();
    reader.read_line(&mut line)?;

    match line.trim_end().split(' ').collect::<Vec<&str>>() {
        parts if parts.len() == 3 => {
            let method = HttpMethod::from_str(parts[0])?;
            let path = parts[1].to_owned();
            let version = parts[2].to_owned();

            Ok((method, path, version))
        }
        parts => Err(RequestParseError::MalformedStartLine(format!(
            "Start line must have exactly three parts, but received {}.",
            parts.len()
        ))),
    }
}

fn parse_query_string(query_string: &str) -> HashMap<String, QueryValue> {
    let mut query: HashMap<String, QueryValue> = HashMap::new();

    if query_string.is_empty() {
        return query;
    }

    for item in query_string.split('&') {
        if let Some((key, val)) = item.split_once('=') {
            if key.ends_with("[]") || key.ends_with("%5B%5D") {
                let key = key
                    .strip_suffix("[]")
                    .or_else(|| key.strip_suffix("%5B%5D"))
                    .unwrap_or(key);

                query
                    .entry(key.to_owned())
                    .and_modify(|e| match e {
                        QueryValue::Many(v) => v.push(val.to_owned()),
                        QueryValue::Single(_) => panic!("Impossible !"),
                    })
                    .or_insert(QueryValue::Many(vec![val.to_owned()]));
            } else {
                query
                    .entry(key.to_owned())
                    .and_modify(|e| match e {
                        QueryValue::Single(_) => {
                            *e = QueryValue::Single(val.to_owned());
                        }
                        QueryValue::Many(_) => panic!("Impossible !"),
                    })
                    .or_insert(QueryValue::Single(val.to_owned()));
            }
        }
    }

    query
}

fn read_headers<T: io::Read>(
    reader: &mut BufReader<T>,
) -> Result<HashMap<String, HeaderValue>, RequestParseError> {
    let mut headers = HashMap::new();
    let mut line = String::new();

    loop {
        line.clear();
        reader.read_line(&mut line)?;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }

        if let Some((key, val)) = trimmed.split_once(": ") {
            let values: HeaderValue = match val.split(", ").collect::<Vec<&str>>() {
                values if values.len() == 1 => HeaderValue::Single(values[0].to_owned()),
                values => HeaderValue::Many(values.iter().map(|str| str.to_string()).collect()),
            };

            headers.insert(key.to_owned(), values);
        }
    }

    Ok(headers)
}

fn get_body<T: io::Read>(
    reader: &mut BufReader<T>,
    method: &HttpMethod,
    headers: &HashMap<String, HeaderValue>,
) -> Result<Option<String>, RequestParseError> {
    if !method.allows_body() {
        Ok(None)
    } else if let Some(value) = headers.get("Transfer-Encoding") {
        Err(RequestParseError::UnsupportedTransferEncoding(
            value.to_string(),
        ))
    } else {
        match headers
            .get("Content-Length")
            .ok_or(RequestParseError::LengthRequired)?
        {
            HeaderValue::Single(length) => {
                let length = length.parse::<usize>()?;

                match read_body_with_content_length(reader, length)? {
                    Some(body) => match headers.get("Content-Type") {
                        Some(content_type) if content_type.to_string().as_str() == "text/plain" => {
                            Ok(Some(body))
                        }
                        Some(_) => Err(RequestParseError::UnsupportedContentType),
                        None => Ok(Some(body)),
                    },
                    None => Ok(None),
                }
            }
            HeaderValue::Many(_) => Err(RequestParseError::InvalidHeaderValue),
        }
    }
}

fn read_body_with_content_length<T: io::Read>(
    reader: &mut BufReader<T>,
    length: usize,
) -> Result<Option<String>, RequestParseError> {
    if length == 0 {
        return Ok(None);
    }

    let mut buf = vec![0u8; length];
    reader.read_exact(&mut buf)?;

    let body = String::from_utf8(buf)?;

    Ok(Some(body))
}

/// All errors that can arise from [Request::parse].
#[derive(Debug)]
pub enum RequestParseError {
    /// An error occurred while reading the HTTP request.
    Read(io::Error),
    /// An error occurred while converting the body `Vec<u8>` to UTF-8.
    InvalidBodyEncoding(FromUtf8Error),
    /// An invalid method was encountered.
    InvalidHttpMethod(InvalidHttpMethodError),
    /// Invalid value for either `Content-Length` or the bytes from a `Transfert-Encoding: chunked` request chunk.
    InvalidSize(ParseIntError),
    /// The `Transfer-Encoding` header is currently unsupported.
    UnsupportedTransferEncoding(String),
    /// The start line is malformed.
    MalformedStartLine(String),
    /// The header `Content-Length` was expected (because the HTTP method requires it) but was not found.
    ///
    /// This can only happen if `Transfer-Encoding` was also unset.
    LengthRequired,
    /// Currently, only `Content-Type: text/plain` is supported.
    UnsupportedContentType,
    /// When an invalid header value is received.
    ///
    /// For example, `Content-Length: 12, 12` is invalid.
    InvalidHeaderValue,
}

impl From<io::Error> for RequestParseError {
    fn from(err: io::Error) -> Self {
        RequestParseError::Read(err)
    }
}

impl From<FromUtf8Error> for RequestParseError {
    fn from(err: FromUtf8Error) -> Self {
        RequestParseError::InvalidBodyEncoding(err)
    }
}

impl From<<HttpMethod as FromStr>::Err> for RequestParseError {
    fn from(err: <HttpMethod as FromStr>::Err) -> Self {
        RequestParseError::InvalidHttpMethod(err)
    }
}

impl From<ParseIntError> for RequestParseError {
    fn from(err: ParseIntError) -> Self {
        RequestParseError::InvalidSize(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_start_line_valid() -> Result<(), RequestParseError> {
        let data = b"GET / HTTP/1.1";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(
            read_start_line(&mut reader)?,
            (HttpMethod::GET, String::from("/"), String::from("HTTP/1.1"))
        );

        Ok(())
    }

    #[test]
    fn test_read_start_line_invalid_method() {
        let data = b"TRY / HTTP/1.1";
        let mut reader = BufReader::new(&data[..]);

        assert!(matches!(
            read_start_line(&mut reader),
            Err(RequestParseError::InvalidHttpMethod(
                InvalidHttpMethodError(_)
            ))
        ));
    }

    #[test]
    fn test_read_start_line_invalid_length() {
        let data = b"HTTP/1.1 / random stuff here GET";
        let mut reader = BufReader::new(&data[..]);

        assert!(matches!(
            read_start_line(&mut reader),
            Err(RequestParseError::MalformedStartLine(_))
        ));
    }

    #[test]
    fn test_read_headers_single_header_valid() -> Result<(), RequestParseError> {
        let data = b"Content-Type: text/plain\r\n\r\n";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(
            read_headers(&mut reader)?,
            HashMap::from([(
                String::from("Content-Type"),
                HeaderValue::Single(String::from("text/plain"))
            )])
        );

        Ok(())
    }

    #[test]
    fn test_read_headers_mutiple_headers_valid() -> Result<(), RequestParseError> {
        let data = b"Accept: application/json\r\n\
                                Content-Type: text/plain\r\n\
                                Content-Length: 11\r\n\
                                \r\n";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(
            read_headers(&mut reader)?,
            HashMap::from([
                (
                    String::from("Accept"),
                    HeaderValue::Single(String::from("application/json"))
                ),
                (
                    String::from("Content-Type"),
                    HeaderValue::Single(String::from("text/plain"))
                ),
                (
                    String::from("Content-Length"),
                    HeaderValue::Single(String::from("11"))
                )
            ])
        );

        Ok(())
    }

    #[test]
    fn test_read_headers_invalid_headers_skipped() -> Result<(), RequestParseError> {
        let data = b"Accept\r\n\
                                Content-Type:text/plain\r\n\
                                Content-Length: 11\r\n\
                                \r\n";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(
            read_headers(&mut reader)?,
            HashMap::from([(
                String::from("Content-Length"),
                HeaderValue::Single(String::from("11"))
            )])
        );

        Ok(())
    }

    #[test]
    fn test_read_body_with_content_length_valid() -> Result<(), RequestParseError> {
        let data = b"Hello World";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(
            read_body_with_content_length(&mut reader, data.len())?,
            Some(String::from_utf8_lossy(data).into_owned())
        );

        Ok(())
    }

    #[test]
    fn test_read_body_with_content_length_zero() -> Result<(), RequestParseError> {
        let data = b"Hello World";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(read_body_with_content_length(&mut reader, 0)?, None);

        Ok(())
    }

    #[test]
    fn test_read_body_with_content_length_not_matching() -> Result<(), RequestParseError> {
        let data = b"Hello World";
        let mut reader = BufReader::new(&data[..]);

        assert!(matches!(
            read_body_with_content_length(&mut reader, data.len() + 3),
            Err(RequestParseError::Read(_))
        ));

        Ok(())
    }
}
