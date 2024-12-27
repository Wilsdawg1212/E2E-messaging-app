use ring::{aead, rand};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey};

pub struct Crypto {
    key: LessSafeKey,
}

impl Crypto {
    pub fn new(key_bytes: &[u8]) -> Self {
        let key = UnboundKey::new(&aead::AES_256_GCM, key_bytes).unwrap();
        Self {
            key: LessSafeKey::new(key),
        }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut ciphertext = plaintext.to_vec();
        let nonce_bytes = [0u8; 12]; // Replace this with a proper random nonce in production
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        self.key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext).unwrap();
        // [nonce.as_ref(), &ciphertext].concat()
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let nonce_bytes = &ciphertext[..12];
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes.try_into().unwrap()).unwrap();
        let ciphertext = &mut ciphertext[12..];
        self.key
            .open_in_place(nonce, Aad::empty(), ciphertext)
            .unwrap()
            .to_vec()

    }
}
