//! Defines the error pattern for the current backend.
//! This file is used in `src/main.rs`.
//! Each module defines its own errors, which are then converted into the `Error` type.
//! This allows errors to be handled using the `Result` type.
//! The `From` trait is used to convert other error types into the `Error` type.
//! The `Debug` trait is used to print errors when using the `?` (Option) operator.
//! The `Display` trait is used to print errors.
//! Each error is defined as an `Error` enum.

use derive_more::From;
use lib_core::model;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    // -- Modules
    #[from]
    Model(model::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
