use crate::certificates::load_certs;
use futures_util::{SinkExt, StreamExt};
use rustls::pki_types::{CertificateDer, ServerName};
use rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::Duration;
use tokio_rustls::TlsConnector;
use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::Message;

pub async fn transmitter(ip: String, port: String) {
    let certs: Vec<CertificateDer> = load_certs("./ssl/cert.pem");

    // Trust our self-signed cert
    let mut root_store = RootCertStore::empty();
    root_store.add_parsable_certificates(certs);
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    let stream = TcpStream::connect(format!("{}:{}", ip, port))
        .await
        .expect("TCP connect failed");

    let server_name = ServerName::try_from("localhost").expect("Invalid server name");

    let tls_stream = match tokio::time::timeout(
        Duration::from_secs(10),
        connector.connect(server_name, stream),
    )
    .await
    {
        Ok(Ok(s)) => {
            println!("TLS handshake successful");
            s
        }
        Ok(Err(e)) => {
            eprintln!("TLS handshake failed: {}", e);
            return;
        }
        Err(_) => {
            eprintln!("TLS handshake timeout");
            return;
        }
    };

    let (socket, _) = client_async(format!("wss://{}:{}/web_socket", ip, port), tls_stream)
        .await
        .expect("WebSocket connect failed");

    println!("Connected to: {}:{}", ip, port);
    println!("Connected to: {}:{}", ip, port);
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
