use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};

const NONCE_LEN: usize = 12;

fn cipher() -> Result<Aes256Gcm, String> {
    let key_hex = std::env::var("TOKEN_ENCRYPTION_KEY")
        .map_err(|_| "missing env TOKEN_ENCRYPTION_KEY".to_string())?;
    let key = hex::decode(key_hex.trim()).map_err(|e| e.to_string())?;
    Aes256Gcm::new_from_slice(&key)
        .map_err(|_| "TOKEN_ENCRYPTION_KEY must be 32 bytes (64 hex chars)".to_string())
}

pub fn encrypt(plain: &str) -> Result<String, String> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ct = cipher()?
        .encrypt(&nonce, plain.as_bytes())
        .map_err(|_| "encryption failed".to_string())?;
    let mut out = nonce.to_vec();
    out.extend(ct);
    Ok(hex::encode(out))
}

pub fn decrypt(data: &str) -> Result<String, String> {
    let raw = hex::decode(data).map_err(|e| e.to_string())?;
    if raw.len() <= NONCE_LEN {
        return Err("ciphertext too short".into());
    }
    let (nonce, ct) = raw.split_at(NONCE_LEN);
    let plain = cipher()?
        .decrypt(Nonce::from_slice(nonce), ct)
        .map_err(|_| "decryption failed (wrong key or corrupted data)".to_string())?;
    String::from_utf8(plain).map_err(|e| e.to_string())
}
