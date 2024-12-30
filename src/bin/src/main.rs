use p2p_sparse_messaging::crypto::Crypto;

fn main() {
    let mut device_a = Crypto::new();
    let mut device_b = Crypto::new();

    // Exchange public keys
    let public_key_a = device_a.public_key().to_vec();
    let public_key_b = device_b.public_key().to_vec();

    // Derive session keys
    device_a.derive_session_key(&public_key_b);
    device_b.derive_session_key(&public_key_a);


}
