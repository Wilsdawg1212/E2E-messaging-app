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
        let nonce = Nonce::assume_unique_for_key([0u8; 12]); // Randomize in production
        let mut ciphertext = plaintext.to_vec();
        self.key.seal_in_place_append_tag(Aad::empty(), &mut ciphertext).unwrap();
        [nonce.as_ref(), &ciphertext].concat()
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let (nonce, ciphertext) = ciphertext.split_at(12);
        self.key
            .open_in_place(Aad::empty(), Nonce::try_assume_unique_for_key(nonce).unwrap(), ciphertext)
            .unwrap()
            .to_vec()
    }
}
