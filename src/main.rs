mod crypto;

use std::env;
use std::io::{self, Write};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [send|receive] [optional: message]", args[0]);
        return;
    }

    let role = &args[1];
    let key = b"anexampleverysecurekey!thisismai"; // 256-bit key
    let crypto = crypto::Crypto::new(key);

    match role.as_str() {
        "send" => {
            if args.len() < 3 {
                eprintln!("Please provide a message to send.");
                return;
            }

            let message = args[2].clone();
            send_message(&crypto, &message).await;
        }
        "receive" => {
            receive_message(&crypto).await;
        }
        _ => {
            eprintln!("Invalid role. Use 'send' or 'receive'.");
        }
    }
}

async fn send_message(crypto: &crypto::Crypto, message: &str) {
    let encrypted = crypto.encrypt(message.as_bytes());
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();

    println!("Sending encrypted message: {:?}", encrypted);
    stream.write_all(&encrypted).await.unwrap();
    println!("Message sent!");
}

async fn receive_message(crypto: &crypto::Crypto) {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening for messages on 127.0.0.1:8080...");

    let (mut stream, _) = listener.accept().await.unwrap();
    let mut buffer = vec![0; 1024];

    let n = stream.read(&mut buffer).await.unwrap();
    let encrypted_message = &buffer[..n];

    println!("Received encrypted message: {:?}", encrypted_message);


    // Convert the encrypted message to a mutable Vec<u8>
    let mut encrypted_message_vec = encrypted_message.to_vec();

    let decrypted_message = crypto.decrypt(&mut encrypted_message_vec);
    println!("Decrypted message: {:?}", String::from_utf8_lossy(&decrypted_message));
}
