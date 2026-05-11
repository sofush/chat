use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce, aead::Aead as _};
use base64::{Engine as _, engine::general_purpose};
use hkdf::Hkdf;
use rand::Rng as _;
use sha2::Sha256;

pub fn derive_key(shared_secret: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, shared_secret);

    let mut key = [0u8; 32];

    hk.expand(b"aes-key", &mut key).expect("hkdf expand");

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
