mod error;
mod scheme_01;

pub use self::error::{Error, Result};
use std::fmt;

use crate::pwd::ContentToHash;

pub const DEFAULT_SCHEME: &str = "01";

/// Dynamic dispatch for the hashing scheme
pub trait Scheme {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String>;

    fn validate(&self, to_hash: &ContentToHash, pwd_ef: &str) -> Result<()>;
}

/// The status of the hashing scheme
#[derive(Debug)]
pub enum SchemeStatus {
    Ok,       // The pwd uses the latest scheme
    Outdated, // The pwd uses an old scheme
}

impl fmt::Display for SchemeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SchemeStatus::Ok => write!(f, "Ok"),
            SchemeStatus::Outdated => write!(f, "Outdated"),
        }
    }
}

/// Get the hashing scheme by name
/// dyn Scheme is a trait object
pub fn get_scheme(scheme_name: &str) -> Result<Box<dyn Scheme>> {
    match scheme_name {
        "01" => Ok(Box::new(scheme_01::Scheme01)),
        _ => Err(Error::SchemeNotFound(scheme_name.to_string())),
    }
}
