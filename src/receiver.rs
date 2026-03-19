use crate::certificates::{load_certs, load_key};
use futures_util::{SinkExt, StreamExt};
use rustls::ServerConfig;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub async fn receiver(receiver_addr: String) {
    let certs = load_certs("./ssl/cert.pem");
    let key = load_key("./ssl/key.pem");

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("Bad server config");

    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(&receiver_addr)
        .await
        .expect("Failed to bind");

    println!("Recipient listening on wss://{}/web_socket", receiver_addr);

    while let Ok((stream, _)) = listener.accept().await {
        let sender_ip = stream.peer_addr().unwrap();
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(stream).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("TLS accept error: {}", e);
                    return;
                }
            };

            let ws_stream = accept_async(tls_stream)
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
