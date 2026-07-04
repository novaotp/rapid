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
    io::{self, BufRead as _, BufReader},
};

use crate::{method::Method, version::Version};

/// An HTTP request.
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: Version,
}

impl Request {
    /// Creates an HTTP request from a reader.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::BufReader;
    /// use rapid_http::{method::Method, request::Request, version::Version};
    ///
    /// # fn try_main() -> Result<(), rapid_http::request::RequestError> {
    /// let data = b"GET / HTTP/1.1\r\n\
    ///              \r\n";
    /// let mut reader = BufReader::new(&data[..]);
    ///
    /// let request = Request::from_reader(&mut reader)?;
    ///
    /// assert_eq!(request.method, Method::GET);
    /// assert_eq!(request.path, String::from("/"));
    /// assert_eq!(request.version, Version::HTTP1_1);
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

        Ok(Request {
            method,
            path,
            version,
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

/// All errors that can arise from [Request::parse].
#[derive(Debug)]
pub enum RequestError {
    /// An error occurred while reading the HTTP request.
    Read(io::Error),
    /// The HTTP version used is not supported OR the request line was improperly formed.
    UnsupportedHttpVersion,
    /// The HTTP method in the request line is not supported.
    InvalidMethod,
}

impl From<io::Error> for RequestError {
    fn from(err: io::Error) -> Self {
        RequestError::Read(err)
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
}
