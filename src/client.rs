use tokio_tungstenite::connect_async;
use tokio::io::{self, AsyncBufReadExt};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:3030/ws";
    let (mut socket, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to the server!");

    // Spawn a task to read incoming messages
    tokio::spawn(async move {
        while let Some(msg) = socket.next().await {
            match msg {
                Ok(msg) if msg.is_text() => {
                    println!("Received: {}", msg.to_text().unwrap());
                }
                _ => break,
            }
        }
    });

    // Read input from the user and send messages
    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Some(Ok(line)) = lines.next_line().await {
        if line == "/quit" {
            break;
        }
        socket.send(line.into()).await.expect("Failed to send message");
    }
}
