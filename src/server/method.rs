use std::str::FromStr;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Method {
    HEAD,
    GET,
    QUERY,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
}

impl FromStr for Method {
    type Err = ();

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
            _ => Err(()),
        }
    }
}
