use p256::{
    ecdh::{EphemeralSecret, SharedSecret},
    elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint},
    EncodedPoint,
};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::hkdf::{Salt, HKDF_SHA256};
use ring::rand::{SecureRandom, SystemRandom};

/// Structure to hold ECC keys and symmetric session key
pub struct Crypto {
    private_key: EphemeralSecret,
    public_key: Vec<u8>,
    session_key: Option<LessSafeKey>,
}

impl Crypto {
    /// Generate a new ECC key pair
    pub fn new() -> Self {
        let private_key = EphemeralSecret::random(&mut rand_core::OsRng);
        let public_key = private_key
            .public_key()
            .to_encoded_point(false) // Uncompressed point
            .as_bytes()
            .to_vec();

        Crypto {
            private_key,
            public_key,
            session_key: None,
        }
    }

    /// Get the public key for this instance
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Derive a shared session key using the peer's public key
    /// Derive a shared session key using the peer's public key
    pub fn derive_session_key(&mut self, peer_public_key: &[u8]) {
        use p256::PublicKey;

        // Decode the peer's public key as a PublicKey type
        let peer_key = EncodedPoint::from_bytes(peer_public_key)
            .and_then(PublicKey::from_encoded_point)
            .expect("Invalid public key");
        let shared_secret = self.private_key.diffie_hellman(&peer_key);

        // Derive symmetric key from shared secret using HKDF
        let session_key = Self::derive_symmetric_key(&shared_secret);
        self.session_key = Some(session_key);
    }


    /// Encrypt a message using the session key
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let session_key = self.session_key.as_ref().expect("Session key not derived");

        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12]; // AES-GCM requires a 96-bit nonce
        SystemRandom::new().fill(&mut nonce_bytes).expect("Nonce generation failed");
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        // Encrypt the plaintext
        let mut ciphertext = plaintext.to_vec(); // Copy plaintext into a mutable vector
        session_key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
            .expect("Encryption failed");

        // Combine the nonce and ciphertext into a single Vec<u8> and return
        let mut result = nonce_bytes.to_vec(); // Start with the nonce
        result.extend_from_slice(&ciphertext); // Append the ciphertext
        result
    }


    /// Decrypt a message using the session key
    pub fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let session_key = self.session_key.as_ref().expect("Session key not derived");

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = ciphertext.split_at(12);
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes.try_into().unwrap()).unwrap();

        let mut ciphertext = ciphertext.to_vec();
        session_key
            .open_in_place(nonce, Aad::empty(), &mut ciphertext)
            .expect("Decryption failed");

        ciphertext
    }

    /// Helper to derive a symmetric key from a shared secret
    fn derive_symmetric_key(shared_secret: &SharedSecret) -> LessSafeKey {
        use ring::hkdf::{Salt, HKDF_SHA256};
        use ring::aead::{AES_256_GCM, UnboundKey};

        let salt = Salt::new(HKDF_SHA256, b"example-salt"); // Replace with protocol-specific salt
        let mut okm = [0u8; 32]; // 256-bit output key material
        salt.extract(shared_secret.as_bytes())
            .expand(&[], &AES_256_GCM) // Pass a reference to AES_256_GCM
            .unwrap()
            .fill(&mut okm)
            .unwrap();

        LessSafeKey::new(UnboundKey::new(&AES_256_GCM, &okm).unwrap())
    }

    /// Helper to generate a secure random nonce
    fn generate_nonce() -> Nonce {
        let mut nonce_bytes = [0u8; 12];
        SystemRandom::new().fill(&mut nonce_bytes).expect("Nonce generation failed");
        Nonce::assume_unique_for_key(nonce_bytes)
    }
}
