mod crypto;

use crypto::Crypto;

fn main() {
    let mut device_a = Crypto::new();
    let mut device_b = Crypto::new();

    // Exchange public keys
    let public_key_a = device_a.public_key().to_vec();
    let public_key_b = device_b.public_key().to_vec();

    // Derive session keys
    device_a.derive_session_key(&public_key_b);
    device_b.derive_session_key(&public_key_a);

    // Device A encrypts a message
    let plaintext = b"Hello, Device B!";
    let encrypted_message = device_a.encrypt(plaintext);
    println!("Encrypted message: {:?}", encrypted_message);

    // Device B decrypts the message
    let decrypted_message = device_b.decrypt(&encrypted_message);
    println!(
        "Decrypted message: {:?}",
        String::from_utf8(decrypted_message).unwrap()
    );
}
