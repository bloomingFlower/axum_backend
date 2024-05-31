pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // Date
    DateFailParse(String),

    // Base64
    FailToB64uDecode,
}

/// Implement the Display trait for the Error enum
impl core::fmt::Display for Error {
    // Print the error by using the Display trait
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

/// Implement the Error trait for the Error enum
impl std::error::Error for Error {}
