use warp::Filter;
use warp::ws::WebSocket;
use futures::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct ServerState {
    clients: Arc<tokio::sync::Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
}


#[derive(Deserialize)]
struct ClientMessage {
    to: String,
    message: String,
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
    let (mut tx, mut rx) = ws.split();
    let (client_tx, mut client_rx) = mpsc::unbounded_channel();

    // Assign a unique ID to the client
    let client_id = uuid::Uuid::new_v4().to_string();
    println!("Client connected: {}", client_id);

    // Add client to state
    {
        let mut clients = state.clients.lock().await;
        clients.insert(client_id.clone(), client_tx);
        broadcast_client_list(&clients, &state).await;
    }

    // Task to send messages to the client
    let send_task = tokio::spawn(async move {
        while let Some(message) = client_rx.recv().await {
            if tx.send(warp::ws::Message::text(message)).await.is_err() {
                break;
            }
        }
    });

    // Task to handle incoming messages from the client
    let recv_task = tokio::spawn({
        let state = state.clone();
        let client_id = client_id.clone();
        async move {
            while let Some(Ok(msg)) = rx.next().await {
                if msg.is_text() {
                    // Parse and relay the message
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(msg.to_str().unwrap()) {
                        let recipient_id = client_msg.to;
                        let outgoing_msg = ServerMessage {
                            from: client_id.clone(),
                            message: client_msg.message,
                        };

                        // Forward the message to the recipient
                        let mut clients = state.clients.lock().await;
                        if let Some(recipient_tx) = clients.get(&recipient_id) {
                            let _ = recipient_tx.send(serde_json::to_string(&outgoing_msg).unwrap());
                        } else {
                            println!("Recipient {} not found", recipient_id);
                        }
                    } else {
                        println!("Invalid message format from {}", client_id);
                    }
                }
            }

            // Remove the client on disconnect
            {
                let mut clients = state.clients.lock().await;
                clients.remove(&client_id);
                broadcast_client_list(&clients, &state).await;
            }
            println!("Client disconnected: {}", client_id);
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => (),
        _ = recv_task => (),
    }
}

/// Broadcast the list of connected clients to all clients
async fn broadcast_client_list(
    clients: &HashMap<String, mpsc::UnboundedSender<String>>,
    state: &ServerState,
) {
    let client_list: Vec<String> = clients.keys().cloned().collect();
    let message = serde_json::to_string(&client_list).unwrap();
    for client_tx in clients.values() {
        let _ = client_tx.send(message.clone());
    }
}
