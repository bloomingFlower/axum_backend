mod error;

pub use self::error::{Error, Result};

use crate::config::load_config;
use crate::utils::b64::{b64u_decode_to_string, b64u_encode};
use crate::utils::time::{now_utc, now_utc_plus_sec_str, parse_utc};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
// In test, derive(PartialEq) is used to compare the Token struct
// PartialEq is a trait that is used to compare the values of two types.
#[cfg_attr(test, derive(PartialEq))]
pub struct Token {
    pub identifier: String, // The identifier of the token
    pub exp: String,        // The expiration date of the token
    pub sign_b64u: String,  // The signature of the token
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(token_str: &str) -> std::result::Result<Self, Self::Err> {
        let splits: Vec<&str> = token_str.split('.').collect();
        if splits.len() != 3 {
            return Err(Error::InvalidFormat);
        }
        let (ident_b64u, exp_b64u, sign_b64u) = (splits[0], splits[1], splits[2]);

        Ok(Self {
            identifier: b64u_decode_to_string(ident_b64u).map_err(|_| Error::CannotDecodeIdent)?,
            exp: b64u_decode_to_string(exp_b64u).map_err(|_| Error::CannotDecodeIdent)?,
            sign_b64u: sign_b64u.to_string(),
        })
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            b64u_encode(&self.identifier),
            b64u_encode(&self.exp),
            self.sign_b64u
        )
    }
}

/// Generate a web token for the user
pub fn generate_web_token(user: &str, salt: Uuid) -> Result<Token> {
    let config = &load_config();
    _generate_token(user, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

/// Validate a web token
pub fn validate_web_token(origin_token: &Token, salt: Uuid) -> Result<()> {
    let config = &load_config();
    _validate_token_sign_and_exp(origin_token, salt, &config.TOKEN_KEY)?;

    Ok(())
}

// region: Private functions
fn _generate_token(identifier: &str, duration_sec: i64, salt: Uuid, key: &[u8]) -> Result<Token> {
    let identifier = identifier.to_string();
    let exp = now_utc_plus_sec_str(duration_sec);
    let sign_b64u = _token_sign_into_b64u(&identifier, &exp, salt, key)?;

    Ok(Token {
        identifier,
        exp,
        sign_b64u,
    })
}

fn _validate_token_sign_and_exp(origin_token: &Token, salt: Uuid, key: &[u8]) -> Result<()> {
    let new_sign_b64u =
        _token_sign_into_b64u(&origin_token.identifier, &origin_token.exp, salt, key)?;

    if new_sign_b64u != origin_token.sign_b64u {
        return Err(Error::SignatureNotMatching);
    }

    let origin_exp = parse_utc(&origin_token.exp).map_err(|_| Error::ExpNotIso)?;
    let now = now_utc();
    if origin_exp < now {
        return Err(Error::Expired);
    }

    Ok(())
}

// Create a token from the identifier, expiration date and salt.
fn _token_sign_into_b64u(identifier: &str, exp: &str, salt: Uuid, key: &[u8]) -> Result<String> {
    let content = format!("{}.{}", b64u_encode(identifier), b64u_encode(exp));
    // Hmac is a struct that represents the HMAC algorithm.
    // b64u encode is not encryption process, it is encoding process.
    // Intialize the Hmac struct with the SHA-512 algorithm and the key.
    let mut hmac_sha512 =
        Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::HmacFailNewFromSlice)?;

    // Add the content and the salt data to the HMAC algorithm.
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    // Finalize the HMAC algorithm and convert the result into bytes.
    let hmac_result = hmac_sha512.finalize().into_bytes();
    let result = b64u_encode(hmac_result);

    Ok(result)
}
// endregion: Private functions

// region: Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load_config;
    use anyhow::Result;
    use std::thread;

    #[test]
    fn test_token_display_ok() -> Result<()> {
        let fx_token_str = "ZngtaWRlbnRpZmllci0wMQ.MjAyNC0wNS0yOFQyMzo1MDowMFo.sign-b64u-encoded";
        let fx_token = Token {
            identifier: "fx-identifier-01".to_string(),
            exp: "2024-05-28T23:50:00Z".to_string(),
            sign_b64u: "sign-b64u-encoded".to_string(),
        };

        assert_eq!(fx_token_str, fx_token.to_string());

        Ok(())
    }

    #[test]
    fn test_token_from_str_ok() -> Result<()> {
        let fx_toekn_str = "ZngtaWRlbnRpZmllci0wMQ.MjAyNC0wNS0yOFQyMzo1MDowMFo.sign-b64u-encoded";
        let fx_token = Token {
            identifier: "fx-identifier-01".to_string(),
            exp: "2024-05-28T23:50:00Z".to_string(),
            sign_b64u: "sign-b64u-encoded".to_string(),
        };

        let token: Token = fx_toekn_str.parse()?;
        assert_eq!(token, fx_token);

        Ok(())
    }

    #[test]
    fn test_validate_web_token_ok() -> Result<()> {
        let fx_user = "user_01";
        let fx_salt = Uuid::parse_str("salt_01").unwrap();
        let fx_duration_sec = 1;
        let token_key = &load_config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        thread::sleep(std::time::Duration::from_millis(10));
        let res = validate_web_token(&fx_token, fx_salt);

        res?;

        Ok(())
    }

    #[test]
    fn test_validate_web_token_expired() -> Result<()> {
        let fx_user = "user_01";
        let fx_salt = Uuid::parse_str("salt_01").unwrap();
        let fx_duration_sec = 1;
        let token_key = &load_config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        thread::sleep(std::time::Duration::from_secs(2));
        let res = validate_web_token(&fx_token, fx_salt);

        assert!(
            matches!(res, Err(Error::Expired)),
            "Expected Expired: {:?}",
            res
        );

        Ok(())
    }
}
