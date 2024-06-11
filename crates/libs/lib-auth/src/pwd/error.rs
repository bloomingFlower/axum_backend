use serde::Serialize;

// Define the Result type for simplicity
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    // Key
    KeyFail,

    // Pwd
    NotMatching,
}

impl core::fmt::Display for Error {
    fn fmt(
        &self, 
        fmt: &mut core::fmt::Formatter
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
