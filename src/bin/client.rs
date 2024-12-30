use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:3030/ws";
    let (socket, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to the server!");

    // Split the WebSocket into writer and reader
    let (mut writer, mut reader) = socket.split();

    // Spawn a task to handle incoming messages
    tokio::spawn(async move {
        while let Some(msg) = reader.next().await {
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

    while let Ok(Some(line)) = lines.next_line().await {
        if line == "/quit" {
            break;
        }
        writer.send(line.into()).await.expect("Failed to send message");
    }
}
