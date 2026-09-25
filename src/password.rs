use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier,
    password_hash::{ rand_core::OsRng, SaltString, },
};
use rand::fill;
use hex::encode;
use sha2::{Digest, Sha256};
use jsonwebtoken::{Header, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use crate::{
    structs::{encrypt::Claims},
    constants::{API_URL, FRONT_URL, JWT_LIFETIME_DAYS},
};

pub fn hash_password(password: &str) -> Result<String, password_hash::Error>
{
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<(), password_hash::Error>
{
    let parsed_hash = PasswordHash::new(&password_hash)?;

    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
}

pub fn gen_token() -> String {
    let mut bytes = [0u8; 32];
    fill(&mut bytes);
    encode(bytes)
}

pub fn hash_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    encode(hash)
}

pub fn gen_jwt(token_id: String, secret: &EncodingKey) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::default(),
        &Claims {
            aud: API_URL.to_string(),
            exp: (Utc::now() + Duration::days(JWT_LIFETIME_DAYS)).timestamp() as usize,
            iss: FRONT_URL.to_string(),
            sub: token_id
        },
        &secret
    )
}

pub fn verify_jwt(token: &str, secret: &DecodingKey) -> Result<Claims, jsonwebtoken::errors::Error> {
    match jsonwebtoken::decode::<Claims>(
        token,
        secret,
        &jsonwebtoken::Validation::default()
    ) {
        Ok(token_data) => Ok(token_data.claims),
        Err(err) => Err(err)
    }
}
