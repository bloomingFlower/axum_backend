use super::{Error, Result};
use crate::crypt::{encrypt_into_b64u, EncryptContent};
use crate::load_config;

/// Encrypt the password.
pub fn encrypt_pwd(enc_content: &EncryptContent) -> Result<String> {
    // Load the password key from the configuration.
    let key = &load_config().PWD_KEY;
    // Encrypt the password.
    let encrypted = encrypt_into_b64u(key, enc_content)?;
    // Return the encrypted password with a key version prefix.
    Ok(format!("#01#{}", encrypted))
}

/// Validate the password.
pub fn validate_pwd(enc_content: &EncryptContent, pwd_ref: &str) -> Result<()> {
    let pwd = encrypt_pwd(enc_content)?;

    if pwd == pwd_ref {
        Ok(())
    } else {
        Err(Error::PwdNotMatching)
    }
}
