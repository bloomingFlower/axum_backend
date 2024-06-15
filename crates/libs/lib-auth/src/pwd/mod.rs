mod error;
mod scheme;

pub use self::error::{Error, Result};
pub use self::scheme::SchemeStatus;

use crate::pwd::scheme::{get_scheme, Scheme, DEFAULT_SCHEME};
use lazy_regex::regex_captures;
use std::str::FromStr;
use uuid::Uuid;

pub struct ContentToHash {
    pub content: String,
    pub salt: Uuid,
}

pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
    hash_for_scheme(DEFAULT_SCHEME, to_hash)
}

pub fn validate_pwd(to_hash: &ContentToHash, pwd_ref: &str) -> Result<SchemeStatus> {
    let PwdParts {
        scheme_name,
        hashed,
    } = pwd_ref.parse()?;

    validate_for_scheme(&scheme_name, to_hash, &hashed)?;

    if scheme_name == DEFAULT_SCHEME {
        Ok(SchemeStatus::Ok)
    } else {
        Ok(SchemeStatus::Outdated)
    }
}

fn hash_for_scheme(scheme_name: &str, to_hash: &ContentToHash) -> Result<String> {
    let scheme = get_scheme(scheme_name)?;
    let pwd_hashed = scheme.hash(to_hash)?;

    Ok(format!("#{}#{}", scheme_name, pwd_hashed))
}

fn validate_for_scheme(scheme_name: &str, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()> {
    let scheme = get_scheme(scheme_name)?;
    scheme.validate(to_hash, pwd_ref)?;
    Ok(())
}

struct PwdParts {
    /// The scheme only
    scheme_name: String,
    /// The hashed password
    hashed: String,
}

/// Parse
impl FromStr for PwdParts {
    type Err = Error;

    fn from_str(pwd_with_scheme: &str) -> Result<Self> {
        regex_captures!(r#"^#(\w+)#(.*)"#, pwd_with_scheme,)
            .map(|(_, scheme, hashed)| Self {
                scheme_name: scheme.to_string(),
                hashed: hashed.to_string(),
            })
            .ok_or(Error::PwdWithSchemeFailedParse)
    }
}

/// cargo watch -q -c -x "test -q -p lib-auth test_multi_scheme -- --nocapture"
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_multi_scheme_ok() -> Result<()> {
        let fx_salt = Uuid::parse_str("e5d87716-65d9-4450-8f59-316ce50962fa")?;
        let fx_to_hash = ContentToHash {
            content: "hello world".to_string(),
            salt: fx_salt,
        };

        let pwd_hashed = hash_for_scheme("01", &fx_to_hash)?;
        let pwd_validate = validate_pwd(&fx_to_hash, &pwd_hashed)?;

        assert!(
            matches!(pwd_validate, SchemeStatus::Outdated),
            "status should be outdated"
        );

        Ok(())
    }
}
