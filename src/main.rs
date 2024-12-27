mod crypto;
mod p2p;
mod fallback;

fn main() {
    let key = b"anexampleverysecurekey!niubibibp"; // 256-bit key (32 bytes)
    let crypto = crypto::Crypto::new(key);

    let plaintext = b"Hello, encrypted world!";
    println!("Plaintext: {:?}", String::from_utf8_lossy(plaintext));

    let mut encrypted = crypto.encrypt(plaintext);
    println!("Encrypted: {:?}", encrypted);

    let decrypted = crypto.decrypt(&mut encrypted);
    println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));

    // #[tokio::main]
    // async fn main() {
    //     // // Example: Start the fallback server
    //     // tokio::spawn(async {
    //     //     fallback::run_server().await;
    //     // });
    //     //
    //     // // Example: Start P2P listening
    //     // p2p::start_p2p_listener().await;
    //
    // }

}
