use futures_util::{SinkExt, StreamExt};
use std::io::Write;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

async fn receiver(receiver_addr: String) {
    let listener = TcpListener::bind(receiver_addr.as_str())
        .await
        .expect("Failed to bind to entry point");
    println!(
        "Recipient listening on ws://{}/web_socket",
        receiver_addr.as_str()
    );

    while let Ok((stream, _)) = listener.accept().await {
        let sender_ip = stream.local_addr().unwrap();
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("WebSocket handshake failed");
            let (mut write, mut read) = ws_stream.split();

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("Received from {}: {}", sender_ip, text);
                        write.send(Message::Text(text)).await.unwrap();
                    }
                    Ok(Message::Binary(bin)) => {
                        println!("Received: {:?}", bin);
                        write.send(Message::Binary(bin)).await.unwrap();
                    }
                    Ok(Message::Close(_)) => {
                        println!("Client disconnected");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        println!("Received a ping");
                        write.send(Message::Pong(data)).await.unwrap(); // Ping should reply with Pong, not echo the Ping back
                    }
                    Ok(Message::Pong(_)) => {}
                    Ok(_) => {} // Handle any other variants to satisfy exhaustiveness
                    Err(e) => {
                        println!("Recipient error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
async fn transmitter(transmitter_addr: String) {
    let (socket, _) = connect_async(transmitter_addr.clone())
        .await
        .expect("Failed to connect");
    println!("Connected to: {}", transmitter_addr);
    let (mut write, mut read) = socket.split();

    let reader = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(msg) => println!("sent: {}", msg),
                Err(e) => {
                    println!("sent error: {}", e);
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let ip = "0.0.0.0";
    let port = read_input("Input Current port: ");

    let receiver_addr = format!("{}:{}", ip, port);

    println!("Receiver address: {}", receiver_addr);
    tokio::spawn(receiver(receiver_addr));

    // Give the server a moment to start
    sleep(Duration::from_millis(100)).await;

    let ip = read_input("Input Current ip: ");
    let port = read_input("Input Current port: ");

    let transmitter_addr = format!("ws://{}:{}/web_socket", ip, port);
    transmitter(transmitter_addr).await;
}
