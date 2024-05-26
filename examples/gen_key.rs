use anyhow::Result;
use rand::RngCore;

fn main() -> Result<()> {
    let mut key = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut key);
    println!("\nGenerated key for HMAC: {:?}", key);

    let b64u = base64_url::encode(&key);
    println!("\nBase64 URL encoded key: {:?}", b64u);

    Ok(())
}
