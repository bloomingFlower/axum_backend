mod error;
mod hmac_hasher;

pub use self::error::{Error, Result};

use crate::auth_config;
use crate::pwd::hmac_hasher::hmac_sha512_hash;
use uuid::Uuid;

pub struct ContentToHash {
    pub content: String, // Clear content.
    pub salt: Uuid,      // Clear salt.
}

/// Hash the password with the default scheme.
pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
    // Load the password key from the configuration.
    let key = &auth_config().PWD_KEY;
    // Encrypt the password.
    let hashed = hmac_sha512_hash(key, to_hash)?;
    // Return the encrypted password with a key version prefix.
    Ok(format!("#01#{hashed}"))
}

/// Validate if an ContentToHash matches.
pub fn validate_pwd(enc_content: &ContentToHash, pwd_ref: &str) -> Result<()> {
    let pwd = hash_pwd(enc_content)?;

    if pwd == pwd_ref {
        Ok(())
    } else {
        Err(Error::NotMatching)
    }
}
