extern crate rsa;

use rsa::{RSAPrivateKey, PaddingScheme, Hash};

pub fn sign(digest: &[u8], pkcs1_private_key: &[u8]) -> Result<Vec<u8>, String> {
    let private_key = RSAPrivateKey::from_pkcs8(pkcs1_private_key)
        .map_err(|e| format!("Failed to parse private key: {}", e))?;
    let padding_scheme = PaddingScheme::PKCS1v15Sign { hash: Some(Hash::SHA2_256) };
    private_key.sign(padding_scheme, digest)
        .map_err(|e| format!("Failed to sign digest: {}", e))
}