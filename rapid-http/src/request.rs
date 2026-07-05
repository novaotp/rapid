/*
 * Copyright 2026 Sajidur Rahman
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{
    assert_matches,
    collections::HashMap,
    io::{self, BufRead as _, BufReader, Read as _},
    num::ParseIntError,
    string::FromUtf8Error,
};

use crate::{header::HeaderValue, method::Method, version::Version};

/// An HTTP request.
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: Version,
    pub headers: HashMap<String, HeaderValue>,
    pub body: Option<String>,
}

impl Request {
    /// Creates an HTTP request from a reader.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::BufReader;
    /// use std::collections::HashMap;
    /// use rapid_http::{
    ///     header::HeaderValue,
    ///     method::Method,
    ///     request::Request,
    ///     version::Version
    /// };
    ///
    /// # fn try_main() -> Result<(), rapid_http::request::RequestError> {
    /// let data = b"POST / HTTP/1.1\r\n\
    ///              Accept: text/plain, application/json\r\n\
    ///              Content-Type: text/plain\r\n\
    ///              Content-Length: 11\r\n\
    ///              \r\n\
    ///              Hello World\r\n\
    ///              \r\n";
    /// let mut reader = BufReader::new(&data[..]);
    ///
    /// let request = Request::from_reader(&mut reader)?;
    ///
    /// assert_eq!(request.method, Method::POST);
    /// assert_eq!(request.path, String::from("/"));
    /// assert_eq!(request.version, Version::HTTP1_1);
    /// assert_eq!(
    ///     request.headers,
    ///     HashMap::from([
    ///         (
    ///             String::from("Accept"),
    ///             HeaderValue::Many(vec![
    ///                 String::from("text/plain"),
    ///                 String::from("application/json")
    ///             ])
    ///         ),
    ///         (
    ///             String::from("Content-Type"),
    ///             HeaderValue::Single(String::from("text/plain"))
    ///         ),
    ///         (
    ///             String::from("Content-Length"),
    ///             HeaderValue::Single(String::from("11"))
    ///         ),
    ///     ])
    /// );
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
    /// See [RequestError] for more details.
    pub fn from_reader<T: io::Read>(reader: &mut BufReader<T>) -> Result<Self, RequestError> {
        let (method, path, version) = read_start_line(reader)?;
        let headers = read_headers(reader)?;
        let body = get_body(reader, &method, &headers)?;

        Ok(Request {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}

fn read_start_line<T: io::Read>(
    reader: &mut BufReader<T>,
) -> Result<(Method, String, Version), RequestError> {
    let mut line = String::new();
    reader.read_line(&mut line)?;

    match line.split_whitespace().collect::<Vec<&str>>().as_slice() {
        [method_str, path_str, version_str] => {
            let method = method_str
                .parse::<Method>()
                .map_err(|_| RequestError::InvalidMethod)?;

            let path = path_str.to_string();

            let version = version_str
                .parse::<Version>()
                .map_err(|_| RequestError::UnsupportedHttpVersion)?;

            Ok((method, path, version))
        }
        _ => Err(RequestError::UnsupportedHttpVersion),
    }
}

fn read_headers<T: io::Read>(
    reader: &mut BufReader<T>,
) -> Result<HashMap<String, HeaderValue>, RequestError> {
    let mut headers = HashMap::new();
    let mut line = String::new();

    loop {
        line.clear();
        reader.read_line(&mut line)?;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }

        if let Some((k, v)) = trimmed.split_once(":") {
            let key = k.trim().to_owned();
            let value = v
                .parse::<HeaderValue>()
                .map_err(|_| RequestError::InvalidHeaderValue)?;

            headers.insert(key, value);
        } else {
            return Err(RequestError::InvalidHeaderValue);
        }
    }

    Ok(headers)
}

fn get_body<T: io::Read>(
    reader: &mut BufReader<T>,
    method: &Method,
    headers: &HashMap<String, HeaderValue>,
) -> Result<Option<String>, RequestError> {
    if !method.allows_body() {
        return Ok(None);
    } else if headers.get("Transfer-Encoding").is_some() {
        // TODO : Supported `Transfer-Encoding`
        return Err(RequestError::InvalidMediaType);
    }

    // TODO : Support other content types
    if !matches!(headers.get("Content-Type"), Some(HeaderValue::Single(val)) if val.as_str() == "text/plain")
    {
        return Err(RequestError::InvalidMediaType);
    }

    let length_header = headers
        .get("Content-Length")
        .ok_or(RequestError::ContentLengthRequired)?;

    let length = match length_header {
        HeaderValue::Single(len) => len.parse::<usize>()?,
        HeaderValue::Many(_) => return Err(RequestError::InvalidHeaderValue),
    };

    read_body_with_content_length(reader, length)
}

fn read_body_with_content_length<T: io::Read>(
    reader: &mut BufReader<T>,
    length: usize,
) -> Result<Option<String>, RequestError> {
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
pub enum RequestError {
    /// An error occurred while reading the HTTP request.
    Read(io::Error),
    /// The HTTP version used is not supported OR the request line was improperly formed.
    UnsupportedHttpVersion,
    /// The HTTP method in the request line is not supported.
    InvalidMethod,
    /// When an invalid header value is received. For example,
    /// - `Content-Length: 12, 12` is invalid because it only accepts one value.
    /// - `Host:` is invalid because there is no  value after the colon.
    /// - `X-Rapid-Server` is invalid because there is no colon.
    InvalidHeaderValue,
    /// The body's encoding was expected to be UTF-8 but received another encoding.
    InvalidBodyEncoding(FromUtf8Error),
    /// Invalid value (not integer) for either `Content-Length` or the bytes from a `Transfert-Encoding: chunked` request chunk.
    InvalidBodyLength(ParseIntError),
    /// The message content format is not supported.
    ///
    /// The problem is due to the request's indicated `Content-Type`, `Content-Encoding` or `Transfer-Encoding`.
    ///
    /// NOTE : Right now, only `Content-Type: text/plain` is supported.
    InvalidMediaType,
    /// The `Content-Length` header was expected (because the HTTP method requires it) but was not found.
    ///
    /// This can only happen if `Transfer-Encoding` was also unset.
    ContentLengthRequired,
}

impl From<io::Error> for RequestError {
    fn from(err: io::Error) -> Self {
        RequestError::Read(err)
    }
}

impl From<FromUtf8Error> for RequestError {
    fn from(err: FromUtf8Error) -> Self {
        RequestError::InvalidBodyEncoding(err)
    }
}

impl From<ParseIntError> for RequestError {
    fn from(err: ParseIntError) -> Self {
        RequestError::InvalidBodyLength(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_start_line_valid() -> Result<(), RequestError> {
        let data = b"GET / HTTP/1.1";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(
            read_start_line(&mut reader)?,
            (Method::GET, String::from("/"), Version::HTTP1_1)
        );

        Ok(())
    }

    #[test]
    fn test_read_start_line_invalid_length() {
        let data = b"HTTP/1.1 /random stuff GET";
        let mut reader = BufReader::new(&data[..]);

        assert_matches!(
            read_start_line(&mut reader),
            Err(RequestError::UnsupportedHttpVersion)
        );
    }

    #[test]
    fn test_read_headers_single_header_valid() -> Result<(), RequestError> {
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
    fn test_read_headers_mutiple_headers_valid() -> Result<(), RequestError> {
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
    fn test_read_headers_with_ows_valid() -> Result<(), RequestError> {
        let data = b"Content-Type:text/plain\r\n\
                                     Content-Length:       11     \r\n\
                                \r\n";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(
            read_headers(&mut reader)?,
            HashMap::from([
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
    fn test_read_headers_empty_value_invalid() {
        let data = b"Content-Type:\r\n\
                                \r\n";
        let mut reader = BufReader::new(&data[..]);

        assert_matches!(
            read_headers(&mut reader),
            Err(RequestError::InvalidHeaderValue)
        );
    }

    #[test]
    fn test_read_headers_with_missing_colon_invalid() {
        let data = b"Content-Type:\r\n\
                                \r\n";
        let mut reader = BufReader::new(&data[..]);

        assert_matches!(
            read_headers(&mut reader),
            Err(RequestError::InvalidHeaderValue)
        );
    }

    #[test]
    fn test_read_body_with_content_length_valid() -> Result<(), RequestError> {
        let data = b"Hello World\r\n\r\n";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(
            read_body_with_content_length(&mut reader, data.len())?,
            Some(String::from_utf8_lossy(data).into_owned())
        );

        Ok(())
    }

    #[test]
    fn test_read_body_with_content_length_zero() -> Result<(), RequestError> {
        let data = b"Hello World\r\n\r\n";
        let mut reader = BufReader::new(&data[..]);

        assert_eq!(read_body_with_content_length(&mut reader, 0)?, None);

        Ok(())
    }

    #[test]
    fn test_read_body_with_content_length_not_matching() -> Result<(), RequestError> {
        let data = b"Hello World\r\n\r\n";
        let mut reader = BufReader::new(&data[..]);

        assert!(matches!(
            read_body_with_content_length(&mut reader, data.len() + 3),
            Err(RequestError::Read(_))
        ));

        Ok(())
    }
}
