use warp::Filter;
use warp::ws::WebSocket;
use futures::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use std::collections::HashMap;

#[derive(Clone)]
struct ServerState {
    clients: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
}

#[tokio::main]
async fn main() {
    let state = ServerState {
        clients: Arc::new(Mutex::new(HashMap::new())),
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

    // Generate a unique client ID
    let client_id = uuid::Uuid::new_v4().to_string();
    state.clients.lock().unwrap().insert(client_id.clone(), client_tx);

    println!("Client connected: {}", client_id);

    // Task to handle incoming messages
    tokio::spawn(async move {
        while let Some(Ok(msg)) = rx.next().await {
            if let Ok(text) = msg.to_str() {
                println!("Received from {}: {}", client_id, text);
            }
        }

        // Remove client on disconnect
        println!("Client disconnected: {}", client_id);
        state.clients.lock().unwrap().remove(&client_id);
    });

    // Task to handle outgoing messages
    tokio::spawn(async move {
        while let Some(message) = client_rx.recv().await {
            if tx.send(warp::ws::Message::text(message)).await.is_err() {
                break;
            }
        }
    });
}
