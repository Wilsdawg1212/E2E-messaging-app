use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use tokio::io::{self, AsyncBufReadExt};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:3030/ws";
    let (socket, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to the server!");

    // Split the WebSocket into writer and reader
    let (mut writer, mut reader) = socket.split();

    // Store the list of connected clients
    let connected_clients = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Spawn a task to handle incoming messages
    let clients_clone = connected_clients.clone();
    tokio::spawn(async move {
        while let Some(msg) = reader.next().await {
            match msg {
                Ok(msg) if msg.is_text() => {
                    let message = msg.to_text().unwrap();
                    if let Ok(client_list) = serde_json::from_str::<Vec<String>>(message) {
                        // Update the connected client list
                        let mut clients = clients_clone.lock().await;
                        *clients = client_list;
                        println!("Connected clients: {:?}", *clients);
                    } else {
                        // Display received message
                        println!("Received: {}", message);
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
            let clients = connected_clients.lock().await; // Use .await to acquire lock
            println!("Connected clients: {:?}", *clients);
        } else if let Some((recipient, message)) = line.split_once(':') {
            let json_msg = json!({ "to": recipient, "message": message });
            writer.send(json_msg.to_string().into()).await.expect("Failed to send message");
        } else {
            println!("Invalid format. Use recipient_id:message or '/list'.");
        }
    }
}
