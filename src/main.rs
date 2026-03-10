use futures_util::{SinkExt, StreamExt};
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (socket, _) = connect_async("ws://127.0.0.1:3000/web_socket")
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
