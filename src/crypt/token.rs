use crate::crypt::{encrypt_into_b64u, EncryptContent, Error, Result};
use crate::load_config;
use crate::utils::{b64u_encode, now_utc, now_utc_plus_sec_str, parse_utc};
use std::fmt::Display;
use tracing::callsite::Identifier;

pub struct Token {
    pub identifier: String, // The identifier of the token
    pub exp: String,        // The expiration date of the token
    pub sign_b64u: String,  // The signature of the token
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token {{ {}.{}.{} }}",
            b64u_encode(&self.identifier),
            b64u_encode(&self.exp),
            &self.sign_b64u
        )
    }
}

pub fn generate_web_token(user: &str, salt: &str) -> Result<Token> {
    let config = &load_config();
    _generate_token(user, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(orgin_token: &Token, salt: &str) -> Result<()> {
    let config = &load_config();
    _validate_token_sign_and_exp(orgin_token, salt, &config.TOKEN_KEY)?;

    Ok(())
}

// region: Private functions
fn _generate_token(identifier: &str, duration_sec: i64, salt: &str, key: &[u8]) -> Result<Token> {
    let identifier = identifier.to_string();
    let exp = now_utc_plus_sec_str(duration_sec);
    let sign_b64u = _token_sign_into_b64u(&identifier, &exp, salt, key)?;

    Ok(Token {
        identifier,
        exp,
        sign_b64u,
    })
}

fn _validate_token_sign_and_exp(origin_token: &Token, salt: &str, key: &[u8]) -> Result<()> {
    let new_sign_b64u =
        _token_sign_into_b64u(&origin_token.identifier, &origin_token.exp, salt, key)?;

    if new_sign_b64u != origin_token.sign_b64u {
        return Err(Error::TokenSignatureNotMatching);
    }

    let origin_exp = parse_utc(&origin_token.exp).map_err(|_| Error::TokenExpNotIso)?;
    let now = now_utc();
    if origin_exp < now {
        return Err(Error::TokenExpired);
    }

    Ok(())
}

// Create a token from the identifier, expiration date and salt.
fn _token_sign_into_b64u(identifier: &str, exp: &str, salt: &str, key: &[u8]) -> Result<String> {
    let content = format!("{}.{}", b64u_encode(identifier), b64u_encode(exp));
    let sign_b64u = encrypt_into_b64u(
        key,
        &EncryptContent {
            content,
            salt: salt.to_string(),
        },
    )?;

    Ok(sign_b64u)
}
// endregion: Private functions

// region: Tests
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_token_display_ok() -> Result<()> {
        let fx_token = Token {
            identifier: "fx-identifier-01".to_string(),
            exp: "2024-05-28T23:50:00Z".to_string(),
            sign_b64u: "sign-b64u-encoded".to_string(),
        };

        println!("--> fx_token: {}", fx_token);

        Ok(())
    }
}
