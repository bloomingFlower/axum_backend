use super::{Error, Result};
use crate::auth_config;
use crate::pwd::scheme::Scheme;
use crate::pwd::ContentToHash;
use hmac::{Hmac, Mac};
use lib_utils::b64::b64u_encode;
use sha2::Sha512;

pub struct Scheme01;

/// Hash the password with the default scheme.
impl Scheme for Scheme01 {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
        // Load the password key from the configuration.
        let key = &auth_config().PWD_KEY;
        // Encrypt the password.
        hash(key, to_hash)
    }

    fn validate(&self, to_hash: &ContentToHash, raw_pwd_ref: &str) -> Result<()> {
        let raw_pwd_new = self.hash(to_hash)?;
        if raw_pwd_new == raw_pwd_ref {
            Ok(())
        } else {
            Err(Error::PwdValidate)
        }
    }
}

fn hash(key: &[u8], to_hash: &ContentToHash) -> Result<String> {
    let ContentToHash { content, salt } = to_hash;

    // Create a HMAC-SHA-512 from key.
    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::Key)?;

    // Add content.
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    // Finalize and b64u encode.
    let hmac_result = hmac_sha512.finalize();

    let result = b64u_encode(hmac_result.into_bytes());

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_scheme_01_hash_into_b64u_ok() -> Result<()> {
        let fx_salt = Uuid::parse_str("e5d87716-65d9-4450-8f59-316ce50962fa").unwrap();
        let fx_key = &auth_config().TOKEN_KEY;
        let fx_to_hash = ContentToHash {
            content: "hello world".to_string(),
            salt: fx_salt,
        };

        // FIXME
        let fx_res = "QYhiKdhKEHhV1tGwIfYMejb6nhBUMXY-n50sJSbv_Zlqq6S1s0doYHpaSP2bbD2J0zeP1d6M5gli3I-0TTSsSg";

        let res = hash(fx_key, &fx_to_hash)?;

        assert_eq!(res, fx_res);

        Ok(())
    }
}
