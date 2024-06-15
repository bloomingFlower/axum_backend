mod error;
mod scheme_01;
mod scheme_02;

pub use self::error::{Error, Result};
use enum_dispatch::enum_dispatch;
use std::fmt;

use crate::pwd::ContentToHash;

/// HMAC
pub const DEFAULT_SCHEME: &str = "02";

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

/// Dynamic dispatch for the hashing scheme
#[enum_dispatch]
pub trait Scheme {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String>;

    fn validate(&self, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()>;
}

#[enum_dispatch(Scheme)]
enum SchemeDispatcher {
    Scheme01(scheme_01::Scheme01),
    Scheme02(scheme_02::Scheme02),
}

/// Get the hashing scheme by name
/// dyn Scheme is a trait object
pub fn get_scheme(scheme_name: &str) -> Result<impl Scheme> {
    match scheme_name {
        // Scheme is trait type
        "01" => Ok(SchemeDispatcher::Scheme01(scheme_01::Scheme01)),
        "02" => Ok(SchemeDispatcher::Scheme02(scheme_02::Scheme02)),
        _ => Err(Error::SchemeNotFound(scheme_name.to_string())),
    }
}
