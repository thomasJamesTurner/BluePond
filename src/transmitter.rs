use futures_util::{SinkExt, StreamExt};
use openssl::ssl::{SslConnector, SslFiletype, SslMethod};
use openssl::x509::X509;
use std::pin::Pin;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::Duration;
use tokio_openssl::SslStream;
use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::Message;

pub async fn transmitter(ip: String, port: String) {
    let mut connector_builder = SslConnector::builder(SslMethod::tls()).unwrap();

    connector_builder
        .set_private_key_file("./ssl/key.pem", SslFiletype::PEM)
        .unwrap();
    connector_builder
        .set_certificate_chain_file("./ssl/cert.pem")
        .unwrap();

    let client_cert = X509::from_pem(&std::fs::read("./ssl/cert.pem").unwrap()).unwrap();
    connector_builder
        .cert_store_mut()
        .add_cert(client_cert)
        .unwrap();

    connector_builder.set_verify(
        openssl::ssl::SslVerifyMode::PEER | openssl::ssl::SslVerifyMode::FAIL_IF_NO_PEER_CERT,
    );

    let connector = connector_builder.build();
    let stream = TcpStream::connect(format!("{}:{}", ip, port))
        .await
        .expect("TCP connect failed");

    let ssl_config = connector
        .configure()
        .unwrap()
        .verify_hostname(false) // Disable hostname verification if using IP
        .into_ssl(&ip) // Try with IP first
        .unwrap();

    let mut tls_stream = SslStream::new(ssl_config, stream).unwrap();

    // timeout only for out going connections so server can go on
    match tokio::time::timeout(Duration::from_secs(10), Pin::new(&mut tls_stream).connect()).await {
        Ok(Ok(_)) => println!("TLS handshake successful"),
        Ok(Err(e)) => {
            eprintln!("TLS handshake failed: {}", e);
            return;
        }
        Err(_) => {
            eprintln!("TLS handshake timeout");
            return;
        }
    }

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
