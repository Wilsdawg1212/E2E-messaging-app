use warp::Filter;
use warp::ws::WebSocket;
use futures::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
struct ServerState {
    clients: Arc<tokio::sync::Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
    names: Arc<tokio::sync::Mutex<HashMap<String, String>>>, // Maps client IDs to display names
    public_keys: Arc<tokio::sync::Mutex<HashMap<String, Vec<u8>>>>, // Maps client IDs to their public keys
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Register { name: String, public_key: Vec<u8> },
    Send { to: String, message: String },
    RequestPublicKey { for_client: String },
}

#[derive(Serialize)]
struct ServerMessage {
    from: String,
    message: String,
}

#[tokio::main]
async fn main() {
    let state = ServerState {
        clients: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        names: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        public_keys: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    };

    let state_filter = warp::any().map(move || state.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(state_filter)
        .map(|ws: warp::ws::Ws, state: ServerState| {
            ws.on_upgrade(move |socket| handle_connection(socket, state))
        });

    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_connection(ws: WebSocket, state: ServerState) {
    let (tx, mut rx) = ws.split();
    let tx = Arc::new(tokio::sync::Mutex::new(tx)); // Wrap tx in Arc<Mutex>
    let (client_tx, mut client_rx) = mpsc::unbounded_channel();

    // Assign a unique ID to the client
    let client_id = uuid::Uuid::new_v4().to_string();
    println!("Client connected: {}", client_id);

    // Spawn a task to forward messages from client_rx to the WebSocket
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        while let Some(message) = client_rx.recv().await {
            let mut tx = tx_clone.lock().await; // Lock tx for sending
            if tx.send(warp::ws::Message::text(message)).await.is_err() {
                println!("Failed to send message to client");
                break; // Exit loop if sending fails
            } else {
                println!("Message sent to client");
            }
        }
    });

    while let Some(Ok(msg)) = rx.next().await {
        if msg.is_text() {
            let text = msg.to_str().unwrap();

            // Parse the message from the client
            match serde_json::from_str::<ClientMessage>(text) {
                Ok(ClientMessage::Register { name, public_key }) => {
                    // Register the client with name and public key
                    {
                        let mut clients = state.clients.lock().await;
                        let mut names = state.names.lock().await;
                        let mut public_keys = state.public_keys.lock().await;

                        clients.insert(client_id.clone(), client_tx.clone());
                        names.insert(client_id.clone(), name.clone());
                        public_keys.insert(client_id.clone(), public_key);

                        broadcast_client_list(&clients, &names).await;
                    }
                    println!("Client registered with name: {}", name);
                }
                Ok(ClientMessage::Send { to, message }) => {
                    // Relay the encrypted message to the recipient
                    let mut clients = state.clients.lock().await;
                    if let Some(recipient_tx) = clients.get(&to) {
                        let outgoing_msg = json!({
                            "from": client_id.clone(),
                            "message": message
                        });
                        if let Err(e) = recipient_tx.send(serde_json::to_string(&outgoing_msg).unwrap()) {
                            println!("Failed to send message to {}: {}", to, e);
                        } else {
                            println!("Message successfully sent from {} to {}", client_id, to);
                        }
                    } else {
                        println!("Recipient {} not found", to);
                    }
                }
                Ok(ClientMessage::RequestPublicKey { for_client }) => {
                    let public_keys = state.public_keys.lock().await;
                    if let Some(public_key) = public_keys.get(&for_client) {
                        let response = json!({
                            "type": "PublicKeyResponse",
                            "client_id": for_client,
                            "public_key": public_key
                        });
                        let mut tx = tx.lock().await; // Lock tx for sending
                        let _ = tx.send(warp::ws::Message::text(response.to_string())).await;
                    } else {
                        println!("Public key for client {} not found", for_client);
                    }
                }
                Err(_) => {
                    println!("Invalid message format from client {}", client_id);
                }
            }
        }
    }

    // Remove the client on disconnect
    {
        let mut clients = state.clients.lock().await;
        let mut names = state.names.lock().await;
        let mut public_keys = state.public_keys.lock().await;

        clients.remove(&client_id);
        names.remove(&client_id);
        public_keys.remove(&client_id);

        broadcast_client_list(&clients, &names).await;
    }
    println!("Client disconnected: {}", client_id);
}



/// Broadcast the list of connected clients with names and IDs
async fn broadcast_client_list(
    clients: &HashMap<String, mpsc::UnboundedSender<String>>,
    names: &HashMap<String, String>,
) {
    let client_list: Vec<(String, String)> = names
        .iter()
        .map(|(id, name)| (id.clone(), name.clone()))
        .collect();

    let message = serde_json::to_string(&client_list).unwrap();
    println!("Broadcasting client list! {}", message);


    for (client_id, client_tx) in clients.iter() {
        println!("Sending to client: {}", client_id);
        if let Err(e) = client_tx.send(message.clone()) {
            println!("Failed to send client list: {}", e);
        }
    }
}



