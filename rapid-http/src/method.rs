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

use std::{assert_matches, str::FromStr};

/// An invalid method was received.
#[derive(Debug)]
pub struct InvalidMethodError;

/// All valid and supported HTTP methods.
#[derive(Debug, PartialEq, Eq)]
pub enum Method {
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

impl FromStr for Method {
    type Err = InvalidMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HEAD" => Ok(Method::HEAD),
            "GET" => Ok(Method::GET),
            "QUERY" => Ok(Method::QUERY),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "PATCH" => Ok(Method::PATCH),
            "DELETE" => Ok(Method::DELETE),
            "OPTIONS" => Ok(Method::OPTIONS),
            "CONNECT" => Ok(Method::CONNECT),
            "TRACE" => Ok(Method::TRACE),
            _ => Err(InvalidMethodError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_parsing() -> Result<(), InvalidMethodError> {
        let cases = [
            ("HEAD", Method::HEAD),
            ("GET", Method::GET),
            ("QUERY", Method::QUERY),
            ("POST", Method::POST),
            ("PUT", Method::PUT),
            ("PATCH", Method::PATCH),
            ("DELETE", Method::DELETE),
            ("OPTIONS", Method::OPTIONS),
            ("CONNECT", Method::CONNECT),
            ("TRACE", Method::TRACE),
        ];

        for (s, expected) in cases {
            assert_eq!(Method::from_str(s)?, expected);
        }

        Ok(())
    }

    #[test]
    fn test_invalid_method() {
        assert_matches!(Method::from_str("INVALID"), Err(InvalidMethodError));
    }
}
