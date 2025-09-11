use argon2::{
    Algorithm, Argon2, ParamsBuilder, Version,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use base64::prelude::*;

/// Creates an Argon2 hasher with the recommended parameters from the OWASP Password Storage Cheat Sheet:
/// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#introduction
fn create_hasher<'key>() -> Argon2<'key> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::default(),
        ParamsBuilder::new()
            .m_cost(19 * 1024) // 19 MiB of memory.
            .t_cost(2) // 2 iterations.
            .p_cost(1) // 1 degree of parallelism.
            .build()
            .unwrap(),
    )
}

/// Hashes a password using Argon2id.
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let hasher = create_hasher();
    let hash = hasher.hash_password(password.as_bytes(), &salt).unwrap();
    hash.to_string()
}

/// Verifies that the given password matches what's stored in the given hash.
pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    let verifier = Argon2::default();
    verifier
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Generates a 32-byte random token and returns it as a base64-encoded string.
pub fn generate_session_token() -> String {
    let mut token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut token_bytes);
    BASE64_STANDARD.encode(token_bytes)
}
