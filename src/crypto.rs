use p256::{
    ecdh::{EphemeralSecret, SharedSecret},
    elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint},
    EncodedPoint,
};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::hkdf::{Salt, HKDF_SHA256};
use ring::rand::{SecureRandom, SystemRandom};

pub struct Crypto {
    private_key: EphemeralSecret,
    public_key: Vec<u8>,
    shared_secret: Option<[u8; 32]>, // Store the shared secret instead of `LessSafeKey`
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
            shared_secret: None,
        }
    }

    /// Get the public key for this instance
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Derive a shared secret using the peer's public key
    pub fn derive_session_key(&mut self, peer_public_key: &[u8]) {
        use p256::PublicKey;

        // Decode the peer's public key as an EncodedPoint
        let peer_encoded = EncodedPoint::from_bytes(peer_public_key).expect("Invalid public key");

        // Convert the EncodedPoint to a PublicKey
        let peer_key = PublicKey::from_encoded_point(&peer_encoded).unwrap();

        // Compute the shared secret
        let shared_secret = self.private_key.diffie_hellman(&peer_key);

        // Convert the GenericArray<u8, U32> into a [u8; 32]
        let mut secret_bytes = [0u8; 32];
        secret_bytes.copy_from_slice(shared_secret.as_bytes());

        // Store the shared secret
        self.shared_secret = Some(secret_bytes);
    }

    /// Get the shared secret for storage or external use
    pub fn get_shared_secret(&self) -> [u8; 32] {
        self.shared_secret.expect("Shared secret not derived")
    }

    /// Create a symmetric encryption key from a shared secret
    pub fn create_symmetric_key(shared_secret: &[u8; 32]) -> LessSafeKey {
        let salt = Salt::new(HKDF_SHA256, b"example-salt");
        let mut okm = [0u8; 32];
        salt.extract(shared_secret)
            .expand(&[], &AES_256_GCM)
            .unwrap()
            .fill(&mut okm)
            .unwrap();

        let unbound_key = UnboundKey::new(&AES_256_GCM, &okm).unwrap();
        LessSafeKey::new(unbound_key)
    }

    /// Encrypt plaintext with a specified symmetric key
    pub fn encrypt_with_key(key: &LessSafeKey, plaintext: &[u8]) -> Vec<u8> {
        let nonce = Self::generate_nonce();
        let nonce_slice = nonce.as_ref().to_vec();

        // Encrypt the plaintext
        let mut ciphertext = plaintext.to_vec();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
            .expect("Encryption failed");

        // Combine nonce and ciphertext
        [nonce_slice, ciphertext].concat()
    }

    /// Decrypt ciphertext with a specified symmetric key
    pub fn decrypt_with_key(key: &LessSafeKey, ciphertext: &[u8]) -> Vec<u8> {
        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = ciphertext.split_at(12);
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes.try_into().unwrap()).unwrap();

        let mut ciphertext = ciphertext.to_vec();
        let plaintext_len = key
            .open_in_place(nonce, Aad::empty(), &mut ciphertext)
            .expect("Decryption failed")
            .len();

        ciphertext.truncate(plaintext_len);
        ciphertext
    }

    /// Generate a secure random nonce
    fn generate_nonce() -> Nonce {
        let mut nonce_bytes = [0u8; 12];
        SystemRandom::new().fill(&mut nonce_bytes).unwrap();
        Nonce::assume_unique_for_key(nonce_bytes)
    }
}
