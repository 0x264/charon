use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct Error {
    pub msg: String,
    pub offset: usize
}

impl Error {
    pub fn new(msg: String, offset: usize) -> Self {
        Self {
            msg,
            offset
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)// todo map offset to line & column
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;