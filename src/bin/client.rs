use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use serde_json::json;
use p2p_sparse_messaging::crypto::{Crypto};
use std::sync::Arc;
use base64;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:3030/ws";

    // Prompt user for their display name
    println!("Enter your display name:");
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    let display_name = if let Ok(Some(line)) = lines.next_line().await {
        line.trim().to_string()
    } else {
        println!("Failed to read display name.");
        return;
    };

    // Initialize crypto and generate key pair
    let mut crypto = Crypto::new();
    let public_key = crypto.public_key().to_vec();

    let (socket, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to the server!");

    // Split the WebSocket into writer and reader
    let (mut writer, mut reader) = socket.split();

    // Send the display name and public key to the server
    writer
        .send(json!({ "type": "Register", "name": display_name, "public_key": public_key }).to_string().into())
        .await
        .expect("Failed to send display name and public key");

    // Store connected clients and their shared secrets
    let connected_clients: Arc<tokio::sync::Mutex<Vec<(String, String)>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let shared_secrets = Arc::new(tokio::sync::Mutex::new(HashMap::<String, [u8; 32]>::new()));

    // Spawn a task to handle incoming messages
    let clients_clone = connected_clients.clone();
    let crypto_clone = Arc::new(tokio::sync::Mutex::new(crypto));
    let shared_secrets_clone = shared_secrets.clone();

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
                    }

                    if let Ok(public_key_response) = serde_json::from_str::<serde_json::Value>(message) {
                        if public_key_response["type"] == "PublicKeyResponse" {
                            let peer_id = public_key_response["client_id"].as_str().unwrap();
                            let peer_public_key = public_key_response["public_key"].as_array().unwrap();

                            let peer_public_key_bytes: Vec<u8> = peer_public_key.iter().map(|v| v.as_u64().unwrap() as u8).collect();

                            let mut crypto = crypto_clone.lock().await;
                            crypto.derive_session_key(&peer_public_key_bytes);

                            let mut shared_secrets = shared_secrets_clone.lock().await;
                            shared_secrets.insert(peer_id.to_string(), crypto.get_shared_secret());

                            println!("Shared secret established with client: {}", peer_id);
                        }
                    } else if let Ok(encrypted_message) = base64::decode(message) {
                        let mut shared_secrets = shared_secrets_clone.lock().await;
                        if let Some(secret) = shared_secrets.get("sender_client_id") {
                            let key = Crypto::create_symmetric_key(secret);
                            let decrypted_message = Crypto::decrypt_with_key(&key, &encrypted_message);
                            println!("Decrypted message: {:?}", String::from_utf8_lossy(&decrypted_message));
                        } else {
                            println!("No shared secret available for sender.");
                        }
                    }
                }
                _ => break,
            }
        }
    });

    // Main loop for user input
    println!("Type '/list' to see connected clients, or '/quit' to exit.");

    while let Ok(Some(line)) = lines.next_line().await {
        if line == "/quit" {
            break;
        } else if line == "/list" {
            let clients = connected_clients.lock().await;
            println!("Connected clients: {:?}", *clients);
        } else if let Some((recipient, message)) = line.split_once(':') {
            // Request the recipient's public key if not already known
            {
                let shared_secrets = shared_secrets.lock().await;
                if !shared_secrets.contains_key(recipient) {
                    writer
                        .send(json!({ "type": "RequestPublicKey", "for_client": recipient }).to_string().into())
                        .await
                        .expect("Failed to request public key");
                    continue;
                }
            }

            // Encrypt the message using the shared secret
            let shared_secrets = shared_secrets.lock().await;
            if let Some(secret) = shared_secrets.get(recipient) {
                let key = Crypto::create_symmetric_key(secret);
                let encrypted_message = Crypto::encrypt_with_key(&key, message.as_bytes());
                let encoded_message = base64::encode(encrypted_message);

                writer
                    .send(json!({ "type": "Send", "to": recipient, "message": encoded_message }).to_string().into())
                    .await
                    .expect("Failed to send message");
            } else {
                println!("No shared secret available for recipient.");
            }
        } else {
            println!("Invalid format. Use recipient_id:message or '/list'.");
        }
    }
}
