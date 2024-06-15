mod error;
mod scheme;

pub use self::error::{Error, Result};
pub use self::scheme::SchemeStatus;

use crate::pwd::scheme::{get_scheme, Scheme, DEFAULT_SCHEME};
use lazy_regex::regex_captures;
use std::str::FromStr;
use uuid::Uuid;

#[cfg_attr(test, derive(Clone))]
pub struct ContentToHash {
    pub content: String,
    pub salt: Uuid,
}

pub async fn hash_pwd(to_hash: ContentToHash) -> Result<String> {
    tokio::task::spawn_blocking(move || hash_for_scheme(DEFAULT_SCHEME, to_hash))
        .await
        .map_err(|_| Error::FailSpawnBlockForHash)?
}

pub async fn validate_pwd(to_hash: ContentToHash, pwd_ref: String) -> Result<SchemeStatus> {
    let PwdParts {
        scheme_name,
        hashed,
    } = pwd_ref.parse()?;

    let scheme_status = if scheme_name == DEFAULT_SCHEME {
        SchemeStatus::Ok
    } else {
        SchemeStatus::Outdated
    };

    tokio::task::spawn_blocking(move || validate_for_scheme(&scheme_name, to_hash, hashed))
        .await
        .map_err(|_| Error::FailSpawnBlockForValidate)??;

    Ok(scheme_status)
}

fn hash_for_scheme(scheme_name: &str, to_hash: ContentToHash) -> Result<String> {
    let scheme = get_scheme(scheme_name)?;
    let pwd_hashed = scheme.hash(&to_hash)?;

    Ok(format!("#{}#{}", scheme_name, pwd_hashed))
}

fn validate_for_scheme(scheme_name: &str, to_hash: ContentToHash, pwd_ref: String) -> Result<()> {
    let scheme = get_scheme(scheme_name)?;
    scheme.validate(&to_hash, &pwd_ref)?;
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

    #[tokio::test]
    async fn test_multi_scheme_ok() -> Result<()> {
        let fx_salt = Uuid::parse_str("e5d87716-65d9-4450-8f59-316ce50962fa")?;
        let fx_to_hash = ContentToHash {
            content: "hello world".to_string(),
            salt: fx_salt,
        };

        let pwd_hashed = hash_for_scheme("01", fx_to_hash.clone())?;
        let pwd_validate = validate_pwd(fx_to_hash.clone(), pwd_hashed).await?;

        assert!(
            matches!(pwd_validate, SchemeStatus::Outdated),
            "status should be outdated"
        );

        Ok(())
    }
}
