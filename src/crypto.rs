use ring::{aead, rand};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey};

pub struct Crypto {
    key: LessSafeKey,
}

impl Crypto {
    pub fn new(key_bytes: &[u8]) -> Self {
        assert_eq!(key_bytes.len(), 32, "Key must be 32 bytes long for AES-256-GCM");
        let key = UnboundKey::new(&aead::AES_256_GCM, key_bytes).unwrap();
        Self {
            key: LessSafeKey::new(key),
        }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut ciphertext = plaintext.to_vec();
        let nonce_bytes = [0u8; 12]; // Replace with proper random nonce in production
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        // Store the nonce bytes before we move the nonce
        let nonce_slice = nonce.as_ref().to_vec();

        // Now we can move nonce into seal_in_place_append_tag
        self.key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext).unwrap();

        // Use the stored nonce_slice for concatenation
        [nonce_slice.as_slice(), ciphertext.as_slice()].concat()
    }

    pub fn decrypt(&self, ciphertext: &mut [u8]) -> Vec<u8> {
        let nonce_bytes = &ciphertext[..12];
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes.try_into().unwrap()).unwrap();
        let ciphertext = &mut ciphertext[12..];
        self.key
            .open_in_place(nonce, Aad::empty(), ciphertext)
            .unwrap()
            .to_vec()

    }
}
