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

use std::str::FromStr;

/// An invalid version was received.
#[derive(Debug)]
pub struct InvalidVersionError;

/// The valid HTTP versions.
#[derive(Debug, PartialEq, Eq)]
pub enum Version {
    HTTP1_1,
}

impl FromStr for Version {
    type Err = InvalidVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(Version::HTTP1_1),
            _ => Err(InvalidVersionError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_from_str() -> Result<(), InvalidVersionError> {
        let cases = [("HTTP/1.1", Version::HTTP1_1)];

        for (s, expected) in cases {
            assert_eq!(s.parse::<Version>()?, expected);
        }

        Ok(())
    }
}
