use futures_util::{SinkExt, StreamExt};
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
                        Ok(msg) => {
                            println!("Server received: {}", msg);
                            // Echo it back
                            write.send(msg).await.unwrap();
                        }
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
    let (socket, _) = connect_async("ws://127.0.0.1:9001/web_socket")
        .await
        .expect("Failed to connect");

    println!("Connected");

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
        let mut count = 0;

        loop {
            let msg = format!("hello {}", count);

            if let Err(e) = write.send(Message::Text(msg.into())).await {
                println!("Send error: {}", e);
                break;
            }

            count += 1;

            sleep(Duration::from_secs(1)).await;
        }
    });

    let _ = tokio::join!(reader, writer);
}
