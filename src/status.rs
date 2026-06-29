use std::fmt;

pub enum Status {
    OK = 200,
}

impl Status {
    pub fn as_string(&self) -> &str {
        match self {
            Self::OK => "OK",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
