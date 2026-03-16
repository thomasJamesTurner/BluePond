use futures_util::{SinkExt, StreamExt};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::pin::Pin;
use tokio::net::TcpListener;
use tokio_openssl::SslStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub async fn receiver(receiver_addr: String) {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    acceptor
        .set_private_key_file("./ssl/key.pem", SslFiletype::PEM)
        .unwrap();

    acceptor
        .set_certificate_chain_file("./ssl/cert.pem") // or set_certificate_file() can use either
        .unwrap();

    let acceptor = acceptor.build();
    let listener = TcpListener::bind(receiver_addr.as_str())
        .await
        .expect("Failed to bind to entry point");

    println!(
        "Recipient listening on wss://{}/web_socket",
        receiver_addr.as_str()
    );

    while let Ok((stream, _)) = listener.accept().await {
        let sender_ip = stream.peer_addr().unwrap();
        let ssl_ctx = openssl::ssl::Ssl::new(acceptor.context()).unwrap();
        let mut ssl_stream = SslStream::new(ssl_ctx, stream).unwrap();

        Pin::new(&mut ssl_stream).accept().await.unwrap();

        tokio::spawn(async move {
            let ws_stream = accept_async(ssl_stream)
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
                        write.send(Message::Pong(data)).await.unwrap(); // Ping should reply with Pong
                    }
                    Ok(Message::Pong(_)) => {}
                    Ok(_) => {} //default
                    Err(e) => {
                        println!("Recipient error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
