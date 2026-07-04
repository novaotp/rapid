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

/// A value for a single header.
///
/// # Errors
///
/// When converting from a [`str`], returns an error if the header value is empty or contains only whitespace.
#[derive(Debug, PartialEq, Eq)]
pub enum HeaderValue {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug)]
pub struct InvalidHeaderValueError;

impl FromStr for HeaderValue {
    type Err = InvalidHeaderValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(InvalidHeaderValueError);
        }

        if let Some((_, _)) = trimmed.split_once(",") {
            let values: Vec<String> = trimmed
                .split(",")
                .map(|val| val.trim().to_string())
                .collect();

            Ok(HeaderValue::Many(values))
        } else {
            Ok(HeaderValue::Single(trimmed.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_value_with_ows_from_str() -> Result<(), InvalidHeaderValueError> {
        let cases = [
            (
                "text/plain",
                HeaderValue::Single(String::from("text/plain")),
            ),
            ("12   ", HeaderValue::Single(String::from("12"))),
            ("   chunked", HeaderValue::Single(String::from("chunked"))),
        ];

        for (s, expected) in cases {
            assert_eq!(s.parse::<HeaderValue>()?, expected);
        }

        Ok(())
    }

    #[test]
    fn test_multiple_values_with_ows_from_str() -> Result<(), InvalidHeaderValueError> {
        let cases = [
            (
                "text/plain, application/json",
                HeaderValue::Many(vec![
                    String::from("text/plain"),
                    String::from("application/json"),
                ]),
            ),
            (
                "   gzip,    deflate   , br   ",
                HeaderValue::Many(vec![
                    String::from("gzip"),
                    String::from("deflate"),
                    String::from("br"),
                ]),
            ),
            (
                "fr         ,en",
                HeaderValue::Many(vec![String::from("fr"), String::from("en")]),
            ),
        ];

        for (s, expected) in cases {
            assert_eq!(s.parse::<HeaderValue>()?, expected);
        }

        Ok(())
    }

    #[test]
    fn test_empty_value_from_str() {
        let cases = ["", "      ", "\n"];

        for case in cases {
            assert_matches!(case.parse::<HeaderValue>(), Err(InvalidHeaderValueError));
        }
    }
}
