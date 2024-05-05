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

pub type Result<T> = std::result::Result<T, Error>;