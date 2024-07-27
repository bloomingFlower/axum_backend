//! Argon2id scheme implementation.
//! If the salt is changed, HMAC does not match but Argon2id does.
//! Because of this, the salt is stored in the pwd_ref.

use super::{Error, Result, Scheme};
use crate::config::auth_config;
use crate::pwd::ContentToHash;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use std::sync::LazyLock;

pub struct Scheme02;

impl Scheme for Scheme02 {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
        let argon2 = get_argon2();

        let salt_b64 = SaltString::encode_b64(to_hash.salt.as_bytes()).map_err(|_| Error::Salt)?;

        let pwd = argon2
            .hash_password(to_hash.content.as_bytes(), &salt_b64)
            .map_err(|_| Error::Hash)?
            .to_string();

        Ok(pwd)
    }

    fn validate(&self, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()> {
        let argon2 = get_argon2();

        let parsed_hash_ref = PasswordHash::new(pwd_ref).map_err(|_| Error::Hash)?;

        argon2
            .verify_password(to_hash.content.as_bytes(), &parsed_hash_ref)
            .map_err(|_| Error::PwdValidate)
    }
}

fn get_argon2() -> &'static Argon2<'static> {
    static INSTANCE: LazyLock<Argon2<'static>> = LazyLock::new(|| {
        let key = &auth_config().PWD_KEY;
        Argon2::new_with_secret(key, Algorithm::Argon2id, Version::V0x13, Params::default())
            .expect("Failed to initialize Argon2")
    });

    &INSTANCE
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use uuid::Uuid;

    #[test]
    fn test_scheme_02_hash_into_b64u_ok() -> Result<()> {
        let fx_to_hash = ContentToHash {
            content: "hello world".to_string(),
            salt: Uuid::parse_str("e5d87716-65d9-4450-8f59-316ce50962fa")?,
        };
        let fx_res = "$argon2id$v=19$m=19456,t=2,p=1$5dh3FmXZRFCPWTFs5Qli+g$c/Xh4RNMLaZfN58CW9pV5ye7sRS0ehFdVEXdOKnYaAo";

        let scheme = Scheme02;
        let res = scheme.hash(&fx_to_hash)?;

        assert_eq!(res, fx_res);

        Ok(())
    }
}
