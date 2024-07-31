use derive_more::From;
use scylla::cql_to_rust::FromRowError;
use scylla::frame::value::SerializeValuesError;
use scylla::transport::errors::NewSessionError;
use scylla::transport::errors::QueryError;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::io;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
    #[from]
    NewSession(#[serde_as(as = "DisplayFromStr")] NewSessionError),

    #[from]
    Query(#[serde_as(as = "DisplayFromStr")] QueryError),

    #[from]
    SerializeValues(#[serde_as(as = "DisplayFromStr")] SerializeValuesError),

    #[from]
    FromRow(#[serde_as(as = "DisplayFromStr")] FromRowError),

    #[from]
    Io(#[serde_as(as = "DisplayFromStr")] io::Error),

    #[from]
    Base64(#[serde_as(as = "DisplayFromStr")] base64::DecodeError),

    ScyllaError(String),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
