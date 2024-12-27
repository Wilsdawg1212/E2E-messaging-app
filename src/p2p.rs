use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn start_p2p_listener() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    println!("P2P listener running on 127.0.0.1:8080");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(stream));
        //maybe add a sleep() here so it doesn't kill my computer?
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = vec![0; 1024];
    stream.read(&mut buffer).await.unwrap();

    // For demo purposes, print the received message
    println!("Received: {:?}", String::from_utf8_lossy(&buffer));

    // Send acknowledgment
    stream.write_all(b"Message received").await.unwrap();
}
