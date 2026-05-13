#![allow(deprecated)]

use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce, aead::Aead as _};
use base64::{Engine as _, engine::general_purpose};
use hkdf::Hkdf;
use rand::Rng as _;
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub fn get_public_key_from_bytes(public_key: &[u8]) -> Option<PublicKey> {
    let Ok(public_key) = general_purpose::STANDARD.decode(public_key) else {
        return None;
    };

    let Ok(public_key): Result<[u8; 32], _> = public_key.try_into() else {
        return None;
    };

    Some(PublicKey::from(public_key))
}

pub fn derive_aes_key(
    secret: EphemeralSecret,
    peer_public_key: &PublicKey,
) -> [u8; 32] {
    let shared_secret = secret.diffie_hellman(peer_public_key);
    let hk = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(b"aes-key", &mut key).unwrap();
    key
}

pub fn encrypt_message(key: &[u8; 32], plaintext: &str) -> (String, String) {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let mut nonce_bytes = [0u8; 12];

    rand::rng().fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();

    (
        general_purpose::STANDARD.encode(nonce_bytes),
        general_purpose::STANDARD.encode(ciphertext),
    )
}

pub fn decrypt_message(
    key: &[u8; 32],
    nonce_b64: &str,
    ciphertext_b64: &str,
) -> Option<String> {
    let cipher = Aes256Gcm::new_from_slice(key).ok()?;
    let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64).ok()?;
    let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64).ok()?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
        .ok()?;

    String::from_utf8(plaintext).ok()
}
