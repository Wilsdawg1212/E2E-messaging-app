use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use serde_json::json;
use p2p_sparse_messaging::crypto;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:3030/ws";

    // Prompt user for their display name
    println!("Enter your display name:");
    let mut input = String::new();
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    if let Ok(Some(line)) = lines.next_line().await {
        input = line;
    }
    let display_name = input.trim().to_string();

    // Initialize crypto and generate key pair
    let mut crypto = crypto::Crypto::new();
    let public_key = crypto.public_key().to_vec(); // Your crypto module's method for retrieving the public key

    let (socket, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to the server!");

    // Split the WebSocket into writer and reader
    let (mut writer, mut reader) = socket.split();

    // Send the display name and public key to the server
    writer
        .send(json!({ "name": display_name, "public_key": public_key }).to_string().into())
        .await
        .expect("Failed to send display name and public key");

    // Store the list of connected clients
    let connected_clients = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Spawn a task to handle incoming messages
    let clients_clone = connected_clients.clone();
    let crypto_clone = Arc::new(tokio::sync::Mutex::new(crypto));
    let crypto_for_messages = crypto_clone.clone();

    tokio::spawn(async move {
        while let Some(msg) = reader.next().await {
            match msg {
                Ok(msg) if msg.is_text() => {
                    let message = msg.to_text().unwrap();

                    // Update client list or process encrypted message
                    if let Ok(client_list) = serde_json::from_str::<Vec<(String, String)>>(message) {
                        // Update the connected client list
                        let mut clients = clients_clone.lock().await;
                        *clients = client_list;
                        println!("Connected clients: {:?}", *clients);
                    } else if let Ok(encrypted_message) = base64::decode(message) {
                        // Decrypt the message
                        let mut crypto = crypto_for_messages.lock().await;
                        let decrypted = crypto.decrypt(&encrypted_message);
                        println!("Decrypted message: {:?}", String::from_utf8_lossy(&decrypted));
                    }
                }
                _ => break,
            }
        }
    });

    // Read input from the user and send messages
    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("Type '/list' to see connected clients, or '/quit' to exit.");

    while let Ok(Some(line)) = lines.next_line().await {
        if line == "/quit" {
            break;
        } else if line == "/list" {
            let clients = connected_clients.lock().await;
            println!("Connected clients: {:?}", *clients);
        } else if let Some((recipient, message)) = line.split_once(':') {
            let json_msg = {
                let mut crypto = crypto_clone.lock().await;
                let encrypted = crypto.encrypt(message.as_bytes());
                let encoded = base64::encode(encrypted);
                json!({ "to": recipient, "message": encoded })
            };
            writer.send(json_msg.to_string().into()).await.expect("Failed to send message");
        } else {
            println!("Invalid format. Use recipient_id:message or '/list'.");
        }
    }
}
