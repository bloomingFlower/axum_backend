use anyhow::Result;
use lib_utils::b64::b64u_encode;
use rand::RngCore;

/// Generate a key for HMAC
fn main() -> Result<()> {
    let mut key = [0u8; 64];
    // Fill the key with random bytes
    rand::thread_rng().fill_bytes(&mut key);
    println!("\nGenerated key for HMAC: {:?}", key);
    // Encode the key in Base64 URL
    let b64u = b64u_encode(key);
    println!("\nBase64 URL encoded key: {:?}", b64u);

    Ok(())
}
