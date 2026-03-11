use futures_util::{SinkExt, StreamExt};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // --- SERVER ---
    tokio::spawn(async move {
        let receiver_ip = "127.0.0.1:9001";
        let listener = TcpListener::bind(receiver_ip)
            .await
            .expect("Failed to bind to entry point");
        println!("Server listening on ws://{}/web_socket", receiver_ip);

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let ws_stream = accept_async(stream)
                    .await
                    .expect("WebSocket handshake failed");
                let (mut write, mut read) = ws_stream.split();

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            println!("Server received: {}", text);
                            write.send(Message::Text(text)).await.unwrap();
                        }
                        Ok(Message::Binary(bin)) => {
                            println!("Server received: {:?}", bin);
                            write.send(Message::Binary(bin)).await.unwrap();
                        }
                        Ok(Message::Close(_)) => {
                            println!("Client disconnected");
                            break;
                        }
                        Ok(Message::Ping(data)) => {
                            println!("Server received ping");
                            write.send(Message::Pong(data)).await.unwrap(); // Ping should reply with Pong, not echo the Ping back
                        }
                        Ok(Message::Pong(_)) => {}
                        Ok(_) => {} // Handle any other variants to satisfy exhaustiveness
                        Err(e) => {
                            println!("Server error: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    });

    // Give the server a moment to start
    sleep(Duration::from_millis(100)).await;
    let sender_ip = "ws://127.0.0.1:9001/web_socket";
    let (socket, _) = connect_async(sender_ip).await.expect("Failed to connect");

    println!("Connected to: {}", sender_ip);

    let (mut write, mut read) = socket.split();

    let reader = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(msg) => println!("Received: {}", msg),
                Err(e) => {
                    println!("Receive error: {}", e);
                    break;
                }
            }
        }
    });

    let writer = tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            write.send(Message::Text(line.into())).await.unwrap();
        }
    });

    let _ = tokio::join!(reader, writer);
}
